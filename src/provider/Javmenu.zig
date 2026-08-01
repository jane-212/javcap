const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const provider = @import("root.zig");
const domain = @import("domain");
const infra = @import("infra");
const media = @import("media");
const zq = @import("zigquery");

const Self = @This();

const provider_name = "Javmenu";
const search_url_fmt = "https://javmenu.com/zh/search?wd={s}";
const normal_match_threshold: f64 = 0.8;

const api_client_options: infra.HttpClient.Options = .{ .interval_ms = 1000, .retry = 3 };
const media_client_options: infra.HttpClient.Options = .{ .retry = 3 };

/// CSS selectors describing the javmenu.com page structure.
const selectors = struct {
    const search_item = "div.video-list-item";
    const search_item_title = "h5.card-title";
    const search_item_link = "a";
    const search_item_cover = "img.lazyload";
    const detail_title = "h1 strong";
    const detail_director = "div.director a";
    const detail_genre = "a.genre";
    const detail_actress = "a.actress";
    const detail_card_row = ".card-body > div";
};

/// Chinese labels used by the detail page meta card.
const labels = struct {
    const premiered = "发佈于:";
    const runtime = "时长:";
    const runtime_unit = "分钟";
};

/// Characters stripped from extracted text fields.
const trim_chars = " \n\r\t";

alloc: Allocator,
io: Io,
client: infra.HttpClient,
mediaClient: infra.HttpClient,

/// A single search result entry, with all strings owned by the caller's allocator.
const SearchCandidate = struct {
    id: []const u8,
    detail_href: []const u8,
    cover_url: []const u8,

    fn deinit(self: *SearchCandidate, alloc: Allocator) void {
        alloc.free(self.id);
        alloc.free(self.detail_href);
        alloc.free(self.cover_url);
    }
};

pub fn init(alloc: Allocator, io: Io) !*Self {
    var client = infra.HttpClient.init(alloc, io, api_client_options);
    errdefer client.deinit();

    var mediaClient = infra.HttpClient.init(alloc, io, media_client_options);
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
    return provider_name;
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
    var candidates: std.ArrayList(SearchCandidate) = .empty;
    defer {
        for (candidates.items) |*candidate| candidate.deinit(alloc);
        candidates.deinit(alloc);
    }

    try self.collectCandidates(self.alloc, key, &candidates);

    for (candidates.items) |candidate| {
        if (!try matches(self.alloc, key, candidate.id)) continue;

        nfo.fanart = try self.fetchImage(nfo.alloc, candidate.cover_url);
        return try alloc.dupe(u8, candidate.detail_href);
    }

    return error.NotFound;
}

fn collectCandidates(self: *Self, alloc: Allocator, key: domain.jav.Key, out: *std.ArrayList(SearchCandidate)) !void {
    const show = try key.show(alloc);
    defer alloc.free(show);

    const url = try searchUrl(alloc, show);
    defer alloc.free(url);

    var document = try self.fetchDocument(url);
    defer document.deinit();

    try parseSearchResults(alloc, &document, out);
}

fn searchUrl(alloc: Allocator, show: []const u8) ![]const u8 {
    return std.fmt.allocPrint(alloc, search_url_fmt, .{show});
}

fn fetchDocument(self: *Self, url: []const u8) !zq.Document {
    const uri = try std.Uri.parse(url);
    var response = try self.client.fetch(self.alloc, .{
        .location = .{ .uri = uri },
    });
    defer response.deinit();
    if (response.status != .ok) return error.StatusNotOk;

    return zq.Document.initFromSlice(self.alloc, response.body);
}

fn parseSearchResults(alloc: Allocator, document: *zq.Document, out: *std.ArrayList(SearchCandidate)) !void {
    const items = try document.find(selectors.search_item);
    var itemIt = items.iterator();
    while (itemIt.next()) |item| {
        const titleSel = try item.find(selectors.search_item_title);
        if (titleSel.len() == 0) continue;
        const item_id = trimText(try titleSel.text());
        if (item_id.len == 0) continue;

        const linkSel = try item.find(selectors.search_item_link);
        const href = linkSel.attr("href") orelse continue;

        const imgSel = try item.find(selectors.search_item_cover);
        const cover = imgSel.attr("data-src") orelse imgSel.attr("src") orelse continue;

        var candidate = SearchCandidate{
            .id = try alloc.dupe(u8, item_id),
            .detail_href = try alloc.dupe(u8, trimWhitespace(href)),
            .cover_url = try alloc.dupe(u8, trimWhitespace(cover)),
        };
        errdefer candidate.deinit(alloc);

        try out.append(alloc, candidate);
    }
}

fn matches(alloc: Allocator, key: domain.jav.Key, itemId: []const u8) !bool {
    return switch (key) {
        .fc2, .jav => try matchCodedKey(alloc, key, itemId),
        .normal => |n| {
            const score = try infra.matcher.jaroWinkler(alloc, n, itemId);
            return score > normal_match_threshold;
        },
    };
}

fn matchCodedKey(alloc: Allocator, key: domain.jav.Key, itemId: []const u8) !bool {
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
}

fn parseDetail(self: *Self, nfo: *domain.Nfo, detail_url: []const u8) !void {
    var document = try self.fetchDocument(detail_url);
    defer document.deinit();

    try parseDetailTitle(nfo, &document);
    try parseDetailDirector(nfo, &document);
    try parseDetailGenres(nfo, &document);
    try parseDetailActresses(nfo, &document);
    try parseDetailMeta(nfo, &document);

    nfo.country = .jp;
}

fn parseDetailTitle(nfo: *domain.Nfo, document: *zq.Document) !void {
    const titleSel = try document.find(selectors.detail_title);
    if (cleanTitle(try titleSel.text())) |title| nfo.title = try nfo.alloc.dupe(u8, title);
}

fn parseDetailDirector(nfo: *domain.Nfo, document: *zq.Document) !void {
    const directorSel = try document.find(selectors.detail_director);
    if (directorSel.len() == 0) return;
    const director = trimText(try directorSel.text());
    nfo.director = try nfo.alloc.dupe(u8, director);
}

fn parseDetailGenres(nfo: *domain.Nfo, document: *zq.Document) !void {
    const genreSel = try document.find(selectors.detail_genre);
    var genreIt = genreSel.iterator();
    while (genreIt.next()) |genreEl| {
        const genre = trimText(try genreEl.text());
        try nfo.genres.append(nfo.alloc, try nfo.alloc.dupe(u8, genre));
    }
}

fn parseDetailActresses(nfo: *domain.Nfo, document: *zq.Document) !void {
    const actressSel = try document.find(selectors.detail_actress);
    var actressIt = actressSel.iterator();
    while (actressIt.next()) |actressEl| {
        const actress = trimText(try actressEl.text());
        if (actress.len == 0) continue;
        const n = try nfo.alloc.dupe(u8, actress);
        errdefer nfo.alloc.free(n);
        try nfo.actresses.append(nfo.alloc, .{ .name = n });
    }
}

fn parseDetailMeta(nfo: *domain.Nfo, document: *zq.Document) !void {
    const cardDivs = try document.find(selectors.detail_card_row);
    var cardIt = cardDivs.iterator();
    while (cardIt.next()) |div| {
        const divText = try div.text();
        if (std.mem.indexOf(u8, divText, labels.premiered) != null) {
            if (parseFieldValue(divText, labels.premiered)) |value| {
                nfo.premiered = try nfo.alloc.dupe(u8, value);
            }
        } else if (std.mem.indexOf(u8, divText, labels.runtime) != null) {
            if (parseFieldValue(divText, labels.runtime)) |value| {
                nfo.runtime = parseRuntime(value);
            }
        }
    }
}

/// Strips the leading id token and the trailing site-name token from the
/// detail title (e.g. "STARS-804 标题 ... 無修正アダルト動画 Javmenu").
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

/// Trims the given slice using the standard text whitespace set.
fn trimText(s: []const u8) []const u8 {
    return std.mem.trim(u8, s, trim_chars);
}

/// Trims the given slice using all ASCII whitespace (used for URLs).
fn trimWhitespace(s: []const u8) []const u8 {
    return std.mem.trim(u8, s, &std.ascii.whitespace);
}

/// Returns the text after `label`, skipping NBSP characters that follow it.
fn parseFieldValue(text: []const u8, label: []const u8) ?[]const u8 {
    const start = std.mem.indexOf(u8, text, label) orelse return null;
    var after = text[start + label.len ..];
    after = trimText(after);
    while (after.len >= 2 and after[0] == 0xC2 and after[1] == 0xA0) {
        after = trimText(after[2..]);
    }
    if (after.len == 0) return null;
    return after;
}

/// Parses a runtime like "85分钟", ignoring the unit when present.
fn parseRuntime(s: []const u8) ?u32 {
    const end = std.mem.indexOf(u8, s, labels.runtime_unit) orelse s.len;
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

test "cleanTitle — drops leading id and trailing site token" {
    const raw = "STARS-804 美奈子 【みなこ】 タイトル - 無修正アダルト動画 Javmenu";
    try std.testing.expectEqualStrings("美奈子 【みなこ】 タイトル - 無修正アダルト動画", cleanTitle(raw).?);
}

test "cleanTitle — returns null when too few tokens" {
    try std.testing.expect(cleanTitle("") == null);
    try std.testing.expect(cleanTitle("STARS-804") == null);
    try std.testing.expect(cleanTitle("STARS-804 タイトル") == null);
}

test "parseRuntime — parses with and without unit" {
    try std.testing.expectEqual(@as(?u32, 85), parseRuntime("85分钟"));
    try std.testing.expectEqual(@as(?u32, 85), parseRuntime("85"));
    try std.testing.expectEqual(@as(?u32, null), parseRuntime("abc"));
}

test "parseFieldValue — extracts value after label, handling nbsp" {
    try std.testing.expectEqualStrings("2023-01-01", parseFieldValue("发佈于:\u{a0}2023-01-01", "发佈于:").?);
    try std.testing.expectEqualStrings("2023-01-01", parseFieldValue("发佈于: 2023-01-01", "发佈于:").?);
    try std.testing.expect(parseFieldValue("发佈于:", "发佈于:") == null);
}

test "parseSearchResults — extracts candidates from html" {
    const alloc = std.testing.allocator;

    const html =
        \\<html><body>
        \\<div class="video-list-item">
        \\  <h5 class="card-title"> STARS-804 </h5>
        \\  <a href="/zh/video/abc123"><img class="lazyload" data-src="http://img.example.com/a.jpg"></a>
        \\</div>
        \\<div class="video-list-item">
        \\  <h5 class="card-title">HEYZO-3905</h5>
        \\  <a href="/zh/video/def456"><img class="lazyload" src="http://img.example.com/b.jpg"></a>
        \\</div>
        \\<div class="video-list-item">
        \\  <h5 class="card-title"></h5>
        \\</div>
        \\</body></html>
    ;

    var document = try zq.Document.initFromSlice(alloc, html);
    defer document.deinit();

    var candidates: std.ArrayList(SearchCandidate) = .empty;
    defer {
        for (candidates.items) |*c| c.deinit(alloc);
        candidates.deinit(alloc);
    }

    try parseSearchResults(alloc, &document, &candidates);

    try std.testing.expectEqual(@as(usize, 2), candidates.items.len);
    try std.testing.expectEqualStrings("STARS-804", candidates.items[0].id);
    try std.testing.expectEqualStrings("/zh/video/abc123", candidates.items[0].detail_href);
    try std.testing.expectEqualStrings("http://img.example.com/a.jpg", candidates.items[0].cover_url);
    try std.testing.expectEqualStrings("HEYZO-3905", candidates.items[1].id);
    try std.testing.expectEqualStrings("/zh/video/def456", candidates.items[1].detail_href);
    try std.testing.expectEqualStrings("http://img.example.com/b.jpg", candidates.items[1].cover_url);
}
