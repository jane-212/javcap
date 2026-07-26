const std = @import("std");
const Allocator = std.mem.Allocator;

const Self = @This();

alloc: Allocator,

title: ?[]const u8 = null,
originalTitle: ?[]const u8 = null,
rating: ?f64 = null,
plot: ?[]const u8 = null,
runtime: ?u32 = null,
mpaa: ?Mpaa = null,
id: ?[]const u8 = null,
genres: std.ArrayList([]const u8),
tags: std.ArrayList([]const u8),
country: ?Country = null,
director: ?[]const u8 = null,
premiered: ?[]const u8 = null,
studio: ?[]const u8 = null,
actresses: std.ArrayList(Actress),

pub const Country = enum {
    cn,
    jp,
    us,
};

pub const Mpaa = enum {
    g,
    pg,
    pg13,
    r,
    nc17,
};

pub const Actress = struct {
    name: []const u8,
    thumb: ?[]const u8 = null,
};

pub fn init(alloc: Allocator) !Self {
    var genres = try std.ArrayList([]const u8).initCapacity(alloc, 8);
    errdefer genres.deinit(alloc);

    var tags = try std.ArrayList([]const u8).initCapacity(alloc, 8);
    errdefer tags.deinit(alloc);

    var actresses = try std.ArrayList(Actress).initCapacity(alloc, 8);
    errdefer actresses.deinit(alloc);

    return .{
        .alloc = alloc,
        .genres = genres,
        .tags = tags,
        .actresses = actresses,
    };
}

pub fn deinit(self: *Self) void {
    if (self.title) |s| self.alloc.free(s);
    if (self.originalTitle) |s| self.alloc.free(s);
    if (self.plot) |s| self.alloc.free(s);
    if (self.id) |s| self.alloc.free(s);
    if (self.director) |s| self.alloc.free(s);
    if (self.premiered) |s| self.alloc.free(s);
    if (self.studio) |s| self.alloc.free(s);

    for (self.genres.items) |g| self.alloc.free(g);
    self.genres.deinit(self.alloc);

    for (self.tags.items) |t| self.alloc.free(t);
    self.tags.deinit(self.alloc);

    for (self.actresses.items) |a| {
        self.alloc.free(a.name);
        if (a.thumb) |t| self.alloc.free(t);
    }
    self.actresses.deinit(self.alloc);

    self.* = undefined;
}

pub fn format(self: *const Self, w: *std.Io.Writer) !void {
    try w.writeAll("<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"yes\" ?>");
    try w.writeAll("<movie>");
    if (self.title) |t| try w.print("<title>{s}</title>", .{t});
    if (self.originalTitle) |t| try w.print("<originaltitle>{s}</originaltitle>", .{t});
    if (self.rating) |r| try w.print("<rating>{:.1}</rating>", .{r});
    if (self.plot) |p| try w.print("<plot>{s}</plot>", .{p});
    if (self.runtime) |r| try w.print("<runtime>{}</runtime>", .{r});
    if (self.mpaa) |m| switch (m) {
        .g => try w.writeAll("<mpaa>G</mpaa>"),
        .pg => try w.writeAll("<mpaa>PG</mpaa>"),
        .pg13 => try w.writeAll("<mpaa>PG-13</mpaa>"),
        .r => try w.writeAll("<mpaa>R</mpaa>"),
        .nc17 => try w.writeAll("<mpaa>NC-17</mpaa>"),
    };
    if (self.id) |i| try w.print("<uniqueid type=\"num\" default=\"true\">{s}</uniqueid>", .{i});
    for (self.genres) |g| try w.print("<genre>{s}</genre>", .{g});
    for (self.tags) |t| try w.print("<tag>{s}</tag>", .{t});
    if (self.country) |c| switch (c) {
        .cn => try w.writeAll("<country>国产</country>"),
        .jp => try w.writeAll("<country>日本</country>"),
        .us => try w.writeAll("<country>欧美</country>"),
    };
    if (self.director) |d| try w.print("<director>{s}</director>", .{d});
    if (self.premiered) |p| try w.print("<premiered>{s}</premiered>", .{p});
    if (self.studio) |s| try w.print("<studio>{s}</studio>", .{s});
    for (self.actresses.items) |a| {
        try w.writeAll("<actor>");
        try w.print("<name>{s}</name>", .{a.name});
        if (a.thumb) |t| try w.print("<thumb>{s}</thumb>", .{t});
        try w.writeAll("</actor>");
    }
    try w.writeAll("</movie>");
}
