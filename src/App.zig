const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const domain = @import("domain");

const Self = @This();

io: Io,
alloc: Allocator,
entries: std.ArrayList(domain.Entry),

pub fn init(alloc: Allocator, io: Io) !Self {
    return .{
        .io = io,
        .alloc = alloc,
        .entries = .initCapacity(alloc, 8),
    };
}

pub fn addSource(source: domain.Source) !void {}

pub fn deinit(self: *Self) void {
    self.entries.deinit(self.alloc);
}
