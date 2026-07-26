const std = @import("std");
const Allocator = std.mem.Allocator;

pub fn trimAll(alloc: Allocator, slice: []const u8, values_to_strip: []const u8) ![]const u8 {
    const trimmed = std.mem.trim(u8, slice, values_to_strip);

    var buffer: std.ArrayList(u8) = .empty;
    defer buffer.deinit(alloc);

    var it = std.mem.tokenizeAny(u8, trimmed, values_to_strip);
    while (it.next()) |part| {
        if (buffer.items.len > 0) try buffer.append(alloc, ' ');
        try buffer.appendSlice(alloc, part);
    }

    return try buffer.toOwnedSlice(alloc);
}
