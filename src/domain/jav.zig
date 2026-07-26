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

    pub fn show(self: *const Key, alloc: Allocator) ![]const u8 {
        return switch (self.*) {
            .jav => |jav| try std.fmt.allocPrint(alloc, "{s}-{s}", .{ jav.id, jav.number }),
            .fc2 => |fc2| try std.fmt.allocPrint(alloc, "FC2-{s}", .{fc2}),
            .normal => |normal| try alloc.dupe(u8, normal),
        };
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
