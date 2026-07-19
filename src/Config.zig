const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const builtin = @import("builtin");
const options = @import("options");

const Self = @This();

pub const Source = struct {
    from: []const u8,
    to: []const u8,
};

pause_after_finish: bool,
sources: []Source,

pub fn init(
    alloc: Allocator,
    io: Io,
    user: []const u8,
) !?Parsed(Self) {
    const path = try configPath(alloc, user);
    defer alloc.free(path);

    const cwd = Io.Dir.cwd();
    const file = cwd.openFile(io, path, .{}) catch |err| switch (err) {
        error.FileNotFound => {
            try generateDefaultConfig(io, path);
            std.debug.print("已自动生成默认配置文件 -> {s}\n", .{path});
            return null;
        },
        else => return err,
    };
    defer file.close(io);

    var buffer: [4 * 1024]u8 = undefined;
    var file_reader = file.reader(io, &buffer);
    const reader = &file_reader.interface;

    var file_writer: Io.Writer.Allocating = .init(alloc);
    defer file_writer.deinit();

    _ = try reader.stream(&file_writer.writer, .unlimited);
    const config_file = file_writer.written();

    const config_file_z = try alloc.dupeSentinel(u8, config_file, 0);
    defer alloc.free(config_file_z);

    const config = try std.zon.parse.fromSliceAlloc(Self, alloc, config_file_z, null, .{});
    errdefer std.zon.parse.free(alloc, config);

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
            std.zon.parse.free(self.alloc, self.value);
        }
    };
}

fn configPath(alloc: Allocator, user: []const u8) ![]u8 {
    const root = switch (builtin.os.tag) {
        .macos => "/Users",
        .windows => "C:\\Users",
        .linux => "/home",
        else => @compileError("current os not support"),
    };

    return try std.fs.path.join(alloc, &.{ root, user, ".config", options.name, "config.zon" });
}

fn generateDefaultConfig(io: Io, path: []const u8) !void {
    const default_config = @embedFile("config.default.zon");

    const cwd = Io.Dir.cwd();
    const file = try cwd.createFile(io, path, .{});
    defer file.close(io);

    var buffer: [4 * 1024]u8 = undefined;
    var file_writer = file.writer(io, &buffer);
    const writer = &file_writer.interface;

    try writer.writeAll(default_config);
    try writer.flush();
}
