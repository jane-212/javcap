const std = @import("std");
const Allocator = std.mem.Allocator;

pub const Key = union(enum) {
    jav: JavKey,
    fc2: []const u8,
    normal: []const u8,

    pub fn deinit(self: *Key, alloc: Allocator) void {
        switch (self.*) {
            .jav => |*jav| jav.deinit(alloc),
            .fc2 => |fc2| alloc.free(fc2),
            .normal => |normal| alloc.free(normal),
        }
        self.* = undefined;
    }
};

pub const JavKey = struct {
    id: []const u8,
    number: []const u8,

    pub fn deinit(self: *JavKey, alloc: Allocator) void {
        alloc.free(self.id);
        alloc.free(self.number);
        self.* = undefined;
    }
};
