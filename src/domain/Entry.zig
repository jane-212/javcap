const std = @import("std");
const Allocator = std.mem.Allocator;
const Source = @import("Source.zig");

const Self = @This();

alloc: Allocator,
type: Source.SourceType,
key: []const u8,
file: []const u8,
path: []const u8,
dest: []const u8,

pub fn init(
    alloc: Allocator,
    t: Source.SourceType,
    name: []const u8,
    file: []const u8,
    dest: []const u8,
    path: []const u8,
) !Self {
    const f = try alloc.dupe(u8, file);
    const p = try alloc.dupe(u8, path);
    const key = try std.ascii.allocUpperString(alloc, name);

    return .{
        .alloc = alloc,
        .type = t,
        .key = key,
        .dest = dest,
        .file = f,
        .path = p,
    };
}

pub fn deinit(self: *Self) void {
    self.alloc.free(self.key);
    self.alloc.free(self.file);
    self.alloc.free(self.path);
    self.* = undefined;
}
