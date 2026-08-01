const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const provider = @import("root.zig");
const domain = @import("domain");
const infra = @import("infra");
const media = @import("media");
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
            return s.search(alloc, key);
        }

        fn nameInner(
            ptr: *anyopaque,
        ) []const u8 {
            const s: *Self = @ptrCast(@alignCast(ptr));
            return s.name();
        }

        fn deinitInner(
            ptr: *anyopaque,
        ) void {
            const s: *Self = @ptrCast(@alignCast(ptr));
            s.deinit();
        }

        const vtable = provider.VTable{
            .search = searchInner,
            .name = nameInner,
            .deinit = deinitInner,
        };
    };

    return .{
        .ptr = self,
        .vtable = &Impl.vtable,
    };
}

pub fn name(_: *Self) []const u8 {
    return "Javmenu";
}

pub fn search(self: *Self, alloc: Allocator, key: domain.jav.Key) !domain.Nfo {
    if (!support(key)) return error.NotSupport;

    var nfo = try domain.Nfo.init(alloc);
    errdefer nfo.deinit();

    const show = try key.show(nfo.alloc);
    errdefer nfo.alloc.free(show);

    nfo.id = show;

    const detail_url = try self.find(self.alloc, key, &nfo);
    defer self.alloc.free(detail_url);

    try self.parseDetail(&nfo, detail_url);

    return nfo;
}

fn find(self: *Self, alloc: Allocator, key: domain.jav.Key, nfo: *domain.Nfo) ![]const u8 {
    const show = try key.show(self.alloc);
    defer self.alloc.free(show);

    const url = try std.fmt.allocPrint(self.alloc, "https://javmenu.com/zh/search?wd={s}", .{show});
    defer self.alloc.free(url);
    const uri = try std.Uri.parse(url);
    var response = try self.client.fetch(self.alloc, .{
        .location = .{ .uri = uri },
    });
    defer response.deinit();
    if (response.status != .ok) return error.StatusNotOk;

    var html = try zq.Document.initFromSlice(self.alloc, response.body);
    defer html.deinit();

    const items = try html.find("div.video-list-item");
    var itemIt = items.iterator();
    while (itemIt.next()) |item| {
        const titleSel = try item.find("h5.card-title");
        if (titleSel.len() == 0) continue;
        const item_id = std.mem.trim(u8, try titleSel.text(), " \n\r\t");
        if (item_id.len == 0) continue;
        if (!try matches(self.alloc, key, item_id)) continue;

        const linkSel = try item.find("a");
        const href = linkSel.attr("href") orelse continue;

        const imgSel = try item.find("img.lazyload");
        const cover = imgSel.attr("data-src") orelse imgSel.attr("src") orelse continue;

        nfo.fanart = try self.fetchImage(nfo.alloc, std.mem.trim(u8, cover, &std.ascii.whitespace));

        return try alloc.dupe(u8, std.mem.trim(u8, href, &std.ascii.whitespace));
    }

    return error.NotFound;
}

fn matches(alloc: Allocator, key: domain.jav.Key, itemId: []const u8) !bool {
    switch (key) {
        .fc2, .jav => {
            var itemKey = try media.KeyParser.parse(alloc, itemId);
            defer itemKey.deinit(alloc);
            return switch (key) {
                .jav => |k| std.meta.activeTag(itemKey) == .jav and
                    std.mem.eql(u8, k.id, itemKey.jav.id) and
                    std.mem.eql(u8, k.number, itemKey.jav.number),
                .fc2 => |f| std.meta.activeTag(itemKey) == .fc2 and
                    std.mem.eql(u8, f, itemKey.fc2),
                else => false,
            };
        },
        .normal => |n| {
            const score = try infra.matcher.jaroWinkler(alloc, n, itemId);
            return score > 0.8;
        },
    }

    return false;
}

fn parseDetail(self: *Self, nfo: *domain.Nfo, detail_url: []const u8) !void {
    const uri = try std.Uri.parse(detail_url);
    var response = try self.client.fetch(self.alloc, .{
        .location = .{ .uri = uri },
    });
    defer response.deinit();
    if (response.status != .ok) return error.StatusNotOk;

    var html = try zq.Document.initFromSlice(self.alloc, response.body);
    defer html.deinit();

    const titleSel = try html.find("h1 strong");
    if (cleanTitle(try titleSel.text())) |t| nfo.title = try nfo.alloc.dupe(u8, t);

    const directorSel = try html.find("div.director a");
    if (directorSel.len() > 0) {
        const director = std.mem.trim(u8, try directorSel.text(), " \n\r\t");
        nfo.director = try nfo.alloc.dupe(u8, director);
    }

    const genreSel = try html.find("a.genre");
    var genreIt = genreSel.iterator();
    while (genreIt.next()) |genreEl| {
        const g = std.mem.trim(u8, try genreEl.text(), " \n\r\t");
        try nfo.genres.append(nfo.alloc, try nfo.alloc.dupe(u8, g));
    }

    const actressSel = try html.find("a.actress");
    var actressIt = actressSel.iterator();
    while (actressIt.next()) |actressEl| {
        const actress = std.mem.trim(u8, try actressEl.text(), " \n\r\t");
        if (actress.len == 0) continue;
        const n = try nfo.alloc.dupe(u8, actress);
        errdefer nfo.alloc.free(n);
        try nfo.actresses.append(nfo.alloc, .{ .name = n });
    }

    const cardDivs = try html.find(".card-body > div");
    var cardIt = cardDivs.iterator();
    while (cardIt.next()) |div| {
        const divText = try div.text();
        if (std.mem.indexOf(u8, divText, "发佈于:") != null) {
            if (parseFieldValue(divText, "发佈于:")) |val| {
                nfo.premiered = try nfo.alloc.dupe(u8, val);
            }
        } else if (std.mem.indexOf(u8, divText, "时长:") != null) {
            if (parseFieldValue(divText, "时长:")) |val| {
                nfo.runtime = parseRuntime(val);
            }
        }
    }

    nfo.country = .jp;
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

fn support(key: domain.jav.Key) bool {
    return switch (key) {
        .fc2 => true,
        .jav => true,
        .normal => false,
    };
}

test "matches — jav item matches by content, not pointer" {
    const alloc = std.testing.allocator;

    var key = try media.KeyParser.parse(alloc, "STARS-804");
    defer key.deinit(alloc);

    try std.testing.expect(try matches(alloc, key, "STARS-804"));
    try std.testing.expect(!try matches(alloc, key, "STARS-805"));
    try std.testing.expect(!try matches(alloc, key, "IPX-144"));
}

test "matches — fc2 item matches by content, not pointer" {
    const alloc = std.testing.allocator;

    var key = try media.KeyParser.parse(alloc, "FC2-12345");
    defer key.deinit(alloc);

    try std.testing.expect(try matches(alloc, key, "FC2-12345"));
    try std.testing.expect(try matches(alloc, key, "FC2-PPV-12345"));
    try std.testing.expect(!try matches(alloc, key, "FC2-12346"));
}
