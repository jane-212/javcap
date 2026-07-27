const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const provider = @import("root.zig");
const domain = @import("domain");
const infra = @import("infra");
const zq = @import("zigquery");

const Self = @This();

alloc: Allocator,
io: Io,
client: infra.HttpClient,
mediaClient: infra.HttpClient,

pub fn init(alloc: Allocator, io: Io) !*Self {
    var client = infra.HttpClient.init(alloc, io, .{ .interval_ms = 1000, .retry = 3 });
    errdefer client.deinit();

    var mediaClient = infra.HttpClient.init(alloc, io, .{ .retry = 3 });
    errdefer mediaClient.deinit();

    const self = try alloc.create(Self);
    errdefer alloc.destroy(self);

    self.* = .{
        .alloc = alloc,
        .io = io,
        .client = client,
        .mediaClient = mediaClient,
    };

    return self;
}

pub fn deinit(self: *Self) void {
    self.client.deinit();
    self.mediaClient.deinit();
    self.alloc.destroy(self);
    self.* = undefined;
}

pub fn asProvider(self: *Self) provider.Provider {
    const Impl = struct {
        fn searchInner(
            ptr: *anyopaque,
            alloc: Allocator,
            key: domain.jav.Key,
        ) !domain.Nfo {
            const s: *Self = @ptrCast(@alignCast(ptr));
            return try s.search(alloc, key);
        }

        fn deinitInner(
            ptr: *anyopaque,
        ) void {
            const s: *Self = @ptrCast(@alignCast(ptr));
            s.deinit();
        }

        const vtable = provider.VTable{
            .search = searchInner,
            .deinit = deinitInner,
        };
    };

    return .{
        .ptr = self,
        .vtable = &Impl.vtable,
    };
}

pub fn search(self: *Self, alloc: Allocator, key: domain.jav.Key) !domain.Nfo {
    var nfo = try domain.Nfo.init(alloc);
    errdefer nfo.deinit();

    const show = try key.show(self.alloc);
    defer self.alloc.free(show);

    const url = try std.fmt.allocPrint(self.alloc, "https://javmenu.com/zh/{s}", .{show});
    defer self.alloc.free(url);

    const uri = try std.Uri.parse(url);
    var response = try self.client.fetch(self.alloc, .{
        .location = .{ .uri = uri },
    });
    defer response.deinit();
    if (response.status != .ok) return error.StatusNotOk;

    const body = response.body;
    var html = try zq.Document.initFromSlice(self.alloc, body);
    defer html.deinit();

    nfo.id = try alloc.dupe(u8, show);

    const titleSel = try html.find("h1 strong");
    if (cleanTitle(try titleSel.text())) |t| nfo.title = try alloc.dupe(u8, t);

    const directorSel = try html.find("div.director a");
    if (directorSel.len() > 0) {
        const director = std.mem.trim(u8, try directorSel.text(), " \n\r\t");
        nfo.director = try alloc.dupe(u8, director);
    }

    const genreSel = try html.find("a.genre");
    var genreIt = genreSel.iterator();
    while (genreIt.next()) |genreEl| {
        const g = std.mem.trim(u8, try genreEl.text(), " \n\r\t");
        try nfo.genres.append(alloc, try alloc.dupe(u8, g));
    }

    const actressSel = try html.find("a.actress");
    var actressIt = actressSel.iterator();
    while (actressIt.next()) |actressEl| {
        const name = std.mem.trim(u8, try actressEl.text(), " \n\r\t");
        if (name.len == 0) continue;
        try nfo.actresses.append(alloc, .{ .name = try alloc.dupe(u8, name) });
    }

    const cardDivs = try html.find(".card-body > div");
    var cardIt = cardDivs.iterator();
    while (cardIt.next()) |div| {
        const divText = try div.text();
        if (std.mem.indexOf(u8, divText, "发佈于:") != null) {
            if (parseFieldValue(divText, "发佈于:")) |val| {
                nfo.premiered = try alloc.dupe(u8, val);
            }
        } else if (std.mem.indexOf(u8, divText, "时长:") != null) {
            if (parseFieldValue(divText, "时长:")) |val| {
                nfo.runtime = parseRuntime(val);
            }
        }
    }

    const ogImage = try html.find("meta[property=\"og:image\"]");
    if (ogImage.len() > 0) {
        if (ogImage.attr("content")) |content| {
            const poster = try self.fetchImage(alloc, std.mem.trim(u8, content, &std.ascii.whitespace));
            nfo.poster = poster;
        }
    }

    const fanartSel = try html.find("a[data-fancybox=\"gallery\"]");
    if (fanartSel.len() > 0) {
        if (fanartSel.attr("href")) |href| {
            const fanart = try self.fetchImage(alloc, std.mem.trim(u8, href, &std.ascii.whitespace));
            nfo.fanart = fanart;
        }
    }

    nfo.country = .jp;

    return nfo;
}

fn cleanTitle(raw: []const u8) ?[]const u8 {
    var it = std.mem.tokenizeAny(u8, raw, " \n\r\t");
    _ = it.next() orelse return null;
    const start = it.next() orelse return null;

    var prev = start;
    var cur = start;
    while (it.next()) |token| {
        prev = cur;
        cur = token;
    }
    if (@intFromPtr(prev.ptr) == @intFromPtr(cur.ptr)) return null;

    const start_idx = @intFromPtr(start.ptr) - @intFromPtr(raw.ptr);
    const end_idx = @intFromPtr(prev.ptr) - @intFromPtr(raw.ptr) + prev.len;
    return raw[start_idx..end_idx];
}

fn parseFieldValue(text: []const u8, label: []const u8) ?[]const u8 {
    const start = std.mem.indexOf(u8, text, label) orelse return null;
    var after = text[start + label.len ..];
    after = std.mem.trim(u8, after, " \n\r\t");
    while (after.len >= 2 and after[0] == 0xC2 and after[1] == 0xA0) {
        after = std.mem.trim(u8, after[2..], " \n\r\t");
    }
    if (after.len == 0) return null;
    return after;
}

fn parseRuntime(s: []const u8) ?u32 {
    const end = std.mem.indexOf(u8, s, "分钟") orelse s.len;
    return std.fmt.parseInt(u32, s[0..end], 10) catch null;
}

fn fetchImage(self: *Self, alloc: Allocator, url: []const u8) ![]const u8 {
    const uri = try std.Uri.parse(url);
    var response = try self.mediaClient.fetch(self.alloc, .{
        .location = .{ .uri = uri },
    });
    defer response.deinit();
    if (response.status != .ok) return error.StatusNotOk;

    const body = try alloc.dupe(u8, response.body);
    errdefer alloc.free(body);

    return body;
}
