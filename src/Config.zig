const std = @import("std");
const Allocator = std.mem.Allocator;
const builtin = @import("builtin");
const options = @import("options");
const Io = std.Io;
const domain = @import("domain");
const storage = domain.storage;

pause_after_finish: bool,
sources: []Source,

pub const Source = struct {
    type: storage.Type,
    from: []const u8,
    to: []const u8,
    exts: []const []const u8,
};

const Self = @This();

pub fn init(alloc: Allocator, io: Io, user: []const u8) !Parsed(Self) {
    const path = try configPath(alloc, user);
    defer alloc.free(path);

    const file = Io.Dir.openFileAbsolute(io, path, .{}) catch |err| switch (err) {
        error.FileNotFound => {
            try generateDefaultConfig(alloc, io, user);
            return error.ConfigNotFound;
        },
        else => return err,
    };
    defer file.close(io);

    var buffer: [4096]u8 = undefined;
    var fileReader = file.reader(io, &buffer);
    const reader = &fileReader.interface;

    const configContent = try reader.allocRemaining(alloc, .unlimited);
    defer alloc.free(configContent);

    const configContentZ = try alloc.dupeSentinel(u8, configContent, 0);
    defer alloc.free(configContentZ);

    const config = try std.zon.parse.fromSlice(Self, alloc, configContentZ, null, .{});
    return .{
        .alloc = alloc,
        .value = config,
    };
}

pub fn Parsed(comptime T: type) type {
    return struct {
        alloc: Allocator,
        value: T,

        pub fn deinit(self: *@This()) void {
            self.alloc.free(self.value);
            self.* = undefined;
        }
    };
}

fn generateDefaultConfig(alloc: Allocator, io: Io, user: []const u8) !void {
    const defaultConfig = @embedFile("config.zon");
    const dir = try configDir(alloc, user);
    defer alloc.free(dir);

    const cwd = Io.Dir.cwd();
    try cwd.createDirPath(io, dir);

    const path = try configPath(alloc, user);
    defer alloc.free(path);

    const file = try Io.Dir.createFileAbsolute(io, path, .{});
    defer file.close(io);

    var buffer: [4096]u8 = undefined;
    var fileWriter = file.writer(io, &buffer);
    const writer = &fileWriter.interface;

    try writer.writeAll(defaultConfig);
    try writer.flush();
}

fn configPath(alloc: Allocator, user: []const u8) ![]const u8 {
    const dir = try configDir(alloc, user);
    defer alloc.free(dir);

    return try std.fs.path.join(alloc, &.{
        dir,
        "config.zon",
    });
}

fn configDir(alloc: Allocator, user: []const u8) ![]const u8 {
    const root = switch (builtin.os.tag) {
        .macos => "/Users",
        .linux => "/home",
        .windows => "C:\\Users",
        else => @compileError("Unsupported OS"),
    };

    return try std.fs.path.join(alloc, &.{
        root,
        user,
        ".config",
        options.name,
    });
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

test "Test the config path is absolute" {
    const alloc = std.testing.allocator;
    const path = try configPath(alloc, "cat");
    defer alloc.free(path);

    try std.testing.expect(std.fs.path.isAbsolute(path));
}

test "Test the config directory is right" {
    const alloc = std.testing.allocator;
    const expected = switch (builtin.os.tag) {
        .macos => "/Users/cat/.config/javcap",
        .linux => "/home/cat/.config/javcap",
        .windows => "C:\\Users\\cat\\.config\\javcap",
        else => @compileError("Unsupported OS"),
    };
    const actual = try configDir(alloc, "cat");
    defer alloc.free(actual);

    try std.testing.expectEqualStrings(expected, actual);
}

test "Test the config directory is absolute" {
    const alloc = std.testing.allocator;
    const dir = try configDir(alloc, "cat");
    defer alloc.free(dir);

    try std.testing.expect(std.fs.path.isAbsolute(dir));
}
