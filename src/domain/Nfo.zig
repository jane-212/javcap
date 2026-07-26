const std = @import("std");
const Allocator = std.mem.Allocator;
const Self = @This();

alloc: Allocator,
title: ?[]const u8 = null,

pub fn init(alloc: Allocator) Self {
    return .{
        .alloc = alloc,
    };
}

pub fn deinit(self: *Self) void {
    if (self.title) |title| self.alloc.free(title);
    self.* = undefined;
}
