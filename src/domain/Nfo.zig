const std = @import("std");
const Allocator = std.mem.Allocator;

const Self = @This();

alloc: Allocator,

title: ?[]const u8 = null,
original_title: ?[]const u8 = null,
rating: ?f64 = null,
plot: ?[]const u8 = null,
runtime: ?u32 = null,
mpaa: ?Mpaa = null,
id: ?[]const u8 = null,
genres: std.ArrayList([]const u8),
tags: std.ArrayList([]const u8),
country: ?[]const u8 = null,
director: ?[]const u8 = null,
premiered: ?[]const u8 = null,
studio: ?[]const u8 = null,
actresses: std.ArrayList(Actress),

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
    if (self.original_title) |s| self.alloc.free(s);
    if (self.plot) |s| self.alloc.free(s);
    if (self.id) |s| self.alloc.free(s);
    if (self.country) |s| self.alloc.free(s);
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
