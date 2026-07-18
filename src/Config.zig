const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;

const Self = @This();

pub fn init(
    alloc: Allocator,
    io: Io,
    path: []const u8,
) !Parsed(Self) {
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
