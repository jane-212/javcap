const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const http = std.http;

const Self = @This();

alloc: Allocator,
io: Io,
client: http.Client,

pub fn init(alloc: Allocator, io: Io) Self {
    const client: http.Client = .{
        .allocator = alloc,
        .io = io,
    };
    errdefer client.deinit();

    return .{
        .alloc = alloc,
        .io = io,
        .client = client,
    };
}

pub fn deinit(self: *Self) void {
    self.client.deinit();
    self.* = undefined;
}
