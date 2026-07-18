const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;

const Self = @This();

io: Io,
alloc: Allocator,

pub fn init() !Self {

}

pub fn deinit(self: *Self) void {
    _ = self;
}
