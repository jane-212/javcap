const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const http = std.http;
const json = std.json;
const provider = @import("root.zig");
const domain = @import("domain");
const infra = @import("infra");
const media = @import("media");

const Self = @This();

const base_url = "https://avsox.click";
const api_path = "/javu/data/api/";
const search_per_page: usize = 60;
const max_pages: usize = 5;
const user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0 Safari/537.36";

const extra_headers = [_]http.Header{
    .{ .name = "Accept", .value = "application/json, text/plain, */*" },
    .{ .name = "Accept-Language", .value = "zh-CN,zh;q=0.9,en;q=0.8" },
    .{ .name = "X-Requested-With", .value = "XMLHttpRequest" },
    .{ .name = "Referer", .value = base_url },
};

alloc: Allocator,
io: Io,
client: infra.HttpClient,
mediaClient: infra.HttpClient,

const MovieHit = struct {
    movieId: []const u8,
    movieFanHao: []const u8,
    title: ?[]const u8,

    fn deinit(self: *MovieHit, alloc: Allocator) void {
        alloc.free(self.movieId);
        alloc.free(self.movieFanHao);
        if (self.title) |t| alloc.free(t);
        self.* = undefined;
    }
};

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
    return "Avsox";
}

pub fn search(self: *Self, alloc: Allocator, key: domain.jav.Key) !domain.Nfo {
    if (!support(key)) return error.NotSupport;

    var nfo = try domain.Nfo.init(alloc);
    errdefer nfo.deinit();

    var hit = try self.find(self.alloc, key);
    defer hit.deinit(self.alloc);

    nfo.id = try nfo.alloc.dupe(u8, hit.movieFanHao);

    try self.parseDetail(&nfo, hit);

    return nfo;
}

fn find(self: *Self, alloc: Allocator, key: domain.jav.Key) !MovieHit {
    const show = try key.show(alloc);
    defer alloc.free(show);

    const url = try std.fmt.allocPrint(alloc, "{s}{s}search", .{ base_url, api_path });
    defer alloc.free(url);
    const uri = try std.Uri.parse(url);

    var page: usize = 1;
    while (page <= max_pages) : (page += 1) {
        var hits: std.ArrayList(MovieHit) = .empty;
        defer {
            for (hits.items) |*h| h.deinit(alloc);
            hits.deinit(alloc);
        }

        const payload = try json.Stringify.valueAlloc(alloc, .{ .{ .search = show, .lang = "cn" }, search_per_page, page }, .{});
        defer alloc.free(payload);

        var response = try self.fetchJson(alloc, uri, payload);
        defer response.deinit();

        try parseSearchResponse(alloc, response.body, &hits);

        for (hits.items) |hit| {
            if (try matches(alloc, key, hit.movieFanHao, hit.title)) {
                return .{
                    .movieId = try alloc.dupe(u8, hit.movieId),
                    .movieFanHao = try alloc.dupe(u8, hit.movieFanHao),
                    .title = if (hit.title) |t| try alloc.dupe(u8, t) else null,
                };
            }
        }

        if (hits.items.len < search_per_page) break;
    }

    return error.NotFound;
}

fn fetchJson(self: *Self, alloc: Allocator, uri: std.Uri, payload: []const u8) !infra.HttpClient.FetchResult {
    return self.client.fetch(alloc, .{
        .location = .{ .uri = uri },
        .method = .POST,
        .payload = payload,
        .headers = .{
            .content_type = .{ .override = "application/json" },
            .user_agent = .{ .override = user_agent },
        },
        .extra_headers = &extra_headers,
    });
}

fn parseSearchResponse(alloc: Allocator, body: []const u8, out: *std.ArrayList(MovieHit)) !void {
    const SearchItem = struct {
        movieId: []const u8 = "",
        movieFanHao: []const u8 = "",
        title: ?[]const u8 = null,
    };
    const SearchResponse = struct {
        code: u32 = 0,
        data: ?[]SearchItem = null,
    };

    var parsed = try json.parseFromSlice(SearchResponse, alloc, body, .{ .ignore_unknown_fields = true, .allocate = .alloc_always });
    defer parsed.deinit();

    if (parsed.value.code != 200) return error.NotFound;
    const items = parsed.value.data orelse return error.NotFound;

    try out.ensureUnusedCapacity(alloc, items.len);
    for (items) |item| {
        if (item.movieId.len == 0 or item.movieFanHao.len == 0) continue;
        out.appendAssumeCapacity(.{
            .movieId = try alloc.dupe(u8, item.movieId),
            .movieFanHao = try alloc.dupe(u8, item.movieFanHao),
            .title = if (item.title) |t| try alloc.dupe(u8, t) else null,
        });
    }
}

fn parseDetail(self: *Self, nfo: *domain.Nfo, hit: MovieHit) !void {
    const url = try std.fmt.allocPrint(self.alloc, "{s}{s}getMovie", .{ base_url, api_path });
    defer self.alloc.free(url);
    const uri = try std.Uri.parse(url);

    const payload = try json.Stringify.valueAlloc(self.alloc, .{ hit.movieId, "cn" }, .{});
    defer self.alloc.free(payload);

    var response = try self.fetchJson(self.alloc, uri, payload);
    defer response.deinit();

    try parseDetailResponse(nfo, response.body);

    if (nfo.poster) |poster_url| {
        const image = try self.fetchImage(nfo.alloc, poster_url);
        nfo.alloc.free(poster_url);
        nfo.poster = image;
    }
    if (nfo.fanart) |fanart_url| {
        const image = try self.fetchImage(nfo.alloc, fanart_url);
        nfo.alloc.free(fanart_url);
        nfo.fanart = image;
    }

    nfo.country = .jp;
}

fn parseDetailResponse(nfo: *domain.Nfo, body: []const u8) !void {
    const Entity = struct {
        studioName: ?[]const u8 = null,
        directorName: ?[]const u8 = null,
    };
    const Genre = struct {
        genreName: ?[]const u8 = null,
    };
    const Star = struct {
        starName: ?[]const u8 = null,
        avatar: ?[]const u8 = null,
    };
    const DetailItem = struct {
        title: ?[]const u8 = null,
        title_ja: ?[]const u8 = null,
        description_cn: ?[]const u8 = null,
        description_ja: ?[]const u8 = null,
        releaseDate: ?[]const u8 = null,
        length: ?u32 = null,
        studio: ?Entity = null,
        director: ?Entity = null,
        genre: ?[]Genre = null,
        star: ?[]Star = null,
        posterSmall: ?[]const u8 = null,
        posterLarge: ?[]const u8 = null,
    };
    const DetailResponse = struct {
        code: u32 = 0,
        data: ?DetailItem = null,
    };

    var parsed = try json.parseFromSlice(DetailResponse, nfo.alloc, body, .{ .ignore_unknown_fields = true, .allocate = .alloc_always });
    defer parsed.deinit();

    if (parsed.value.code != 200) return error.NotFound;
    const item = parsed.value.data orelse return error.NotFound;

    if (item.title) |t| {
        if (t.len > 0) nfo.title = try nfo.alloc.dupe(u8, t);
    }
    if (item.title_ja) |t| {
        if (t.len > 0 and (nfo.title == null or !std.mem.eql(u8, nfo.title.?, t))) {
            nfo.originalTitle = try nfo.alloc.dupe(u8, t);
        }
    }
    var description: ?[]const u8 = null;
    if (item.description_cn) |c| {
        if (c.len > 0) description = c;
    }
    if (description == null) {
        if (item.description_ja) |j| {
            if (j.len > 0) description = j;
        }
    }
    if (description) |d| nfo.plot = try nfo.alloc.dupe(u8, d);
    if (item.releaseDate) |d| {
        if (d.len > 0) nfo.premiered = try nfo.alloc.dupe(u8, d);
    }
    if (item.length) |len| nfo.runtime = len;
    if (item.studio) |s| {
        if (s.studioName) |n| {
            if (n.len > 0) nfo.studio = try nfo.alloc.dupe(u8, n);
        }
    }
    if (item.director) |d| {
        if (d.directorName) |n| {
            if (n.len > 0) nfo.director = try nfo.alloc.dupe(u8, n);
        }
    }
    if (item.genre) |genres| {
        try nfo.genres.ensureUnusedCapacity(nfo.alloc, genres.len);
        for (genres) |g| {
            if (g.genreName) |n| {
                if (n.len > 0) nfo.genres.appendAssumeCapacity(try nfo.alloc.dupe(u8, n));
            }
        }
    }
    if (item.star) |stars| {
        try nfo.actresses.ensureUnusedCapacity(nfo.alloc, stars.len);
        for (stars) |st| {
            const star_name = st.starName orelse continue;
            if (star_name.len == 0) continue;
            var actress = domain.Nfo.Actress{ .name = try nfo.alloc.dupe(u8, star_name) };
            if (st.avatar) |a| {
                if (a.len > 0) {
                    if (std.mem.startsWith(u8, a, "http")) {
                        actress.thumb = try nfo.alloc.dupe(u8, a);
                    } else {
                        actress.thumb = try std.fmt.allocPrint(nfo.alloc, "{s}{s}", .{ base_url, a });
                    }
                }
            }
            nfo.actresses.appendAssumeCapacity(actress);
        }
    }
    if (item.posterSmall) |u| {
        if (u.len > 0) nfo.poster = try nfo.alloc.dupe(u8, u);
    }
    if (item.posterLarge) |u| {
        if (u.len > 0) nfo.fanart = try nfo.alloc.dupe(u8, u);
    }
}

fn matches(alloc: Allocator, key: domain.jav.Key, fanhao: []const u8, title: ?[]const u8) !bool {
    switch (key) {
        .fc2, .jav => {
            var itemKey = try media.KeyParser.parse(alloc, fanhao);
            defer itemKey.deinit(alloc);
            return switch (key) {
                .jav => |j| std.meta.activeTag(itemKey) == .jav and
                    std.mem.eql(u8, j.id, itemKey.jav.id) and
                    std.mem.eql(u8, j.number, itemKey.jav.number),
                .fc2 => |f| std.meta.activeTag(itemKey) == .fc2 and
                    std.mem.eql(u8, f, itemKey.fc2),
                else => false,
            };
        },
        .normal => |n| {
            if (asciiEql(n, fanhao)) return true;
            const score = try infra.matcher.jaroWinkler(alloc, n, fanhao);
            if (score > 0.8) return true;
            if (title) |t| {
                const titleScore = try infra.matcher.jaroWinkler(alloc, n, t);
                if (titleScore > 0.8) return true;
            }
            return false;
        },
    }

    return false;
}

fn asciiEql(a: []const u8, b: []const u8) bool {
    if (a.len != b.len) return false;
    for (a, b) |x, y| {
        if (std.ascii.toLower(x) != std.ascii.toLower(y)) return false;
    }
    return true;
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
        .jav => false,
        .normal => false,
    };
}

test "matches — fc2 item matches by content, not pointer" {
    const alloc = std.testing.allocator;

    var key = try media.KeyParser.parse(alloc, "FC2-12345");
    defer key.deinit(alloc);

    try std.testing.expect(try matches(alloc, key, "FC2-PPV-12345", null));
    try std.testing.expect(!try matches(alloc, key, "FC2-PPV-12346", null));
    try std.testing.expect(!try matches(alloc, key, "HEYZO-3905", null));
}

test "matches — jav item matches by content, not pointer" {
    const alloc = std.testing.allocator;

    var key = try media.KeyParser.parse(alloc, "HEYZO-3905");
    defer key.deinit(alloc);

    try std.testing.expect(try matches(alloc, key, "HEYZO-3905", null));
    try std.testing.expect(!try matches(alloc, key, "HEYZO-3906", null));
    try std.testing.expect(!try matches(alloc, key, "STARS-804", null));
}

test "matches — normal matches exact fanhao, rejects unrelated" {
    const alloc = std.testing.allocator;

    var key = try media.KeyParser.parse(alloc, "080918_002");
    defer key.deinit(alloc);

    try std.testing.expect(try matches(alloc, key, "080918_002", null));
    try std.testing.expect(!try matches(alloc, key, "n2046", null));
}

test "matches — normal fuzzy matches similar fanhao or title" {
    const alloc = std.testing.allocator;

    var key = try media.KeyParser.parse(alloc, "080918_002");
    defer key.deinit(alloc);

    try std.testing.expect(try matches(alloc, key, "080918_003", null));
}

test "support — accepts all key kinds" {
    const alloc = std.testing.allocator;

    var fc2Key = try media.KeyParser.parse(alloc, "FC2-12345");
    defer fc2Key.deinit(alloc);
    var javKey = try media.KeyParser.parse(alloc, "HEYZO-3905");
    defer javKey.deinit(alloc);
    var normalKey = try media.KeyParser.parse(alloc, "080918_002");
    defer normalKey.deinit(alloc);

    try std.testing.expect(support(fc2Key));
    try std.testing.expect(support(javKey));
    try std.testing.expect(support(normalKey));
}

test "parseSearchResponse — extracts movie hits" {
    const alloc = std.testing.allocator;

    const body =
        \\{"code":200,"data":[
        \\  {"movieId":"nbqxzlp","movieFanHao":"FC2-PPV-4520684","from":"fc2","title":"t1","length":80},
        \\  {"movieId":"zzz","movieFanHao":"HEYZO-3905","from":"heyzo","title":"t2"}
        \\]}
    ;

    var hits: std.ArrayList(MovieHit) = .empty;
    defer {
        for (hits.items) |*h| h.deinit(alloc);
        hits.deinit(alloc);
    }

    try parseSearchResponse(alloc, body, &hits);

    try std.testing.expectEqual(@as(usize, 2), hits.items.len);
    try std.testing.expectEqualStrings("nbqxzlp", hits.items[0].movieId);
    try std.testing.expectEqualStrings("FC2-PPV-4520684", hits.items[0].movieFanHao);
    try std.testing.expectEqualStrings("t1", hits.items[0].title.?);
    try std.testing.expectEqualStrings("HEYZO-3905", hits.items[1].movieFanHao);
}

test "parseDetailResponse — fills nfo from canned detail json" {
    const alloc = std.testing.allocator;

    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();

    const body =
        \\{"code":200,"data":{
        \\"movieId":"kardvgn","movieFanHao":"n1024","from":"tokyohot",
        \\"title":"【生ドル】","title_ja":"【生ドル】タイトル",
        \\"description_cn":"","description_ja":"説明文",
        \\"releaseDate":"2024-08-21","length":80,
        \\"studio":{"studioId":"x","studioName":"東京熱","studioName_ja":"東京熱"},
        \\"director":null,
        \\"genre":[{"genreId":"a","genreName":"凌辱"},{"genreId":"b","genreName":"同性恋","genreName_ja":"レズ"}],
        \\"star":[{"starId":"s1","starName":"美奈子","avatar":"/tokyohot/media/cast/1.jpg"},{"starId":"s2","starName":"Miho","avatar":"https://cdn.example.com/a.jpg"}],
        \\"posterSmall":"https://file.netcdn.space/storage/fc2ppv/1/ps.jpg",
        \\"posterLarge":"https://file.netcdn.space/storage/fc2ppv/1/pl.jpg"
        \\}}
    ;

    try parseDetailResponse(&nfo, body);

    try std.testing.expectEqualStrings("【生ドル】", nfo.title.?);
    try std.testing.expectEqualStrings("【生ドル】タイトル", nfo.originalTitle.?);
    try std.testing.expectEqualStrings("説明文", nfo.plot.?);
    try std.testing.expectEqualStrings("2024-08-21", nfo.premiered.?);
    try std.testing.expectEqual(@as(?u32, 80), nfo.runtime);
    try std.testing.expectEqualStrings("東京熱", nfo.studio.?);
    try std.testing.expect(nfo.director == null);
    try std.testing.expectEqual(@as(usize, 2), nfo.genres.items.len);
    try std.testing.expectEqualStrings("凌辱", nfo.genres.items[0]);
    try std.testing.expectEqualStrings("同性恋", nfo.genres.items[1]);
    try std.testing.expectEqual(@as(usize, 2), nfo.actresses.items.len);
    try std.testing.expectEqualStrings("美奈子", nfo.actresses.items[0].name);
    try std.testing.expectEqualStrings("https://avsox.click/tokyohot/media/cast/1.jpg", nfo.actresses.items[0].thumb.?);
    try std.testing.expectEqualStrings("https://cdn.example.com/a.jpg", nfo.actresses.items[1].thumb.?);
    try std.testing.expectEqualStrings("https://file.netcdn.space/storage/fc2ppv/1/ps.jpg", nfo.poster.?);
    try std.testing.expectEqualStrings("https://file.netcdn.space/storage/fc2ppv/1/pl.jpg", nfo.fanart.?);
}
