const std = @import("std");
const Allocator = std.mem.Allocator;

const Self = @This();

pub const Actor = struct {
    name: ?[]const u8 = null,
    role: ?[]const u8 = null,
    order: u32 = 0,
    thumb: ?[]const u8 = null,
    type: ?[]const u8 = null,

    pub fn deinit(self: *Actor, alloc: Allocator) void {
        if (self.name) |v| alloc.free(v);
        if (self.role) |v| alloc.free(v);
        if (self.thumb) |v| alloc.free(v);
        if (self.type) |v| alloc.free(v);
        self.* = undefined;
    }
};

pub const UniqueId = struct {
    type: ?[]const u8 = null,
    value: ?[]const u8 = null,
    default: bool = false,

    pub fn deinit(self: *UniqueId, alloc: Allocator) void {
        if (self.type) |v| alloc.free(v);
        if (self.value) |v| alloc.free(v);
        self.* = undefined;
    }
};

alloc: Allocator,

title: ?[]const u8 = null,
originaltitle: ?[]const u8 = null,
plot: ?[]const u8 = null,

runtime: ?u32 = null,

rating: ?[]const u8 = null,
mpaa: ?[]const u8 = null,

genres: ?[][]const u8 = null,
tags: ?[][]const u8 = null,
country: ?[]const u8 = null,
director: ?[]const u8 = null,
studio: ?[]const u8 = null,

actors: ?[]Actor = null,

premiered: ?[]const u8 = null,

trailer: ?[]const u8 = null,

set: ?[]const u8 = null,

uniqueids: ?[]UniqueId = null,

pub fn init(alloc: Allocator) Self {
    return .{ .alloc = alloc };
}

pub fn deinit(self: *Self) void {
    inline for (.{
        "title", "originaltitle", "plot",
        "rating", "mpaa",
        "premiered", "trailer", "set",
        "country", "director", "studio",
    }) |field_name| {
        if (@field(self, field_name)) |v| self.alloc.free(v);
    }

    inline for (.{
        "genres", "tags",
    }) |field_name| {
        if (@field(self, field_name)) |slice| {
            for (slice) |item| self.alloc.free(item);
            self.alloc.free(slice);
        }
    }

    if (self.actors) |actors| {
        for (actors) |*a| a.deinit(self.alloc);
        self.alloc.free(actors);
    }

    if (self.uniqueids) |uniqueids| {
        for (uniqueids) |*u| u.deinit(self.alloc);
        self.alloc.free(uniqueids);
    }

    self.* = undefined;
}
