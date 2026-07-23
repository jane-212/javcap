const std = @import("std");
const Allocator = std.mem.Allocator;
const builtin = @import("builtin");
const options = @import("options");

const Self = @This();

pub fn init(alloc: Allocator, user: []const u8) !Self {
    const path = try configPath(alloc, user);
    defer alloc.free(path);
}

fn configPath(alloc: Allocator, user: []const u8) ![]const u8 {
    const root = switch (builtin.os.tag) {
        .macos => "/Users",
        .linux => "/home",
        .windows => "C:\\Users",
        else => @compileError("Unsupported OS"),
    };

    return try std.fs.path.join(alloc, &.{ root, user, ".config", options.name, "config.zon" });
}

test "Test the config path is right" {
    const alloc = std.testing.allocator;
    const expected = switch (builtin.os.tag) {
        .macos => "/Users/cat/.config/javcap/config.zon",
        .linux => "/home/cat/.config/javcap/config.zon",
        .windows => "C:\\Users\\cat\\.config\\javcap\\config.zon",
        else => @compileError("Unsupported OS"),
    };
    const actual = try configPath(alloc, "cat");
    defer alloc.free(actual);

    try std.testing.expectEqualStrings(expected, actual);
}
