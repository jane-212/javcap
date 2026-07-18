const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const builtin = @import("builtin");
const options = @import("options");

const Self = @This();

pub fn init(
    alloc: Allocator,
    io: Io,
) !Parsed(Self) {
    const path = try config_path(alloc);
    defer alloc.free(path);

    const cwd = std.Io.Dir.cwd();
    const file = try cwd.openFile(io, path, .{});
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

fn config_path(alloc: Allocator) ![]u8 {
    const key = switch (builtin.os.tag) {
        .windows => "USERNAME",
        .macos => "USER",
        .linux => "USER",
        else => @compileError("current os not support"),
    };
    const user = try std.process.Environ.getAlloc(.empty, alloc, key);
    defer alloc.free(user);
    if (user.len == 0) return error.UserNotFound;

    const root = switch (builtin.os.tag) {
        .macos => "/Users",
        .windows => "C:\\Users",
        .linux => "/home",
        else => @compileError("current os not support"),
    };

    return try std.fs.path.join(alloc, &.{ root, user, ".config", options.name, "config.zon" });
}
