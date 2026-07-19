const std = @import("std");
const Io = std.Io;
const builtin = @import("builtin");
const Config = @import("Config.zig");

pub fn main(init: std.process.Init) !void {
    const alloc = init.gpa;
    const io = init.io;
    const env = init.environ_map;

    const key = switch (builtin.os.tag) {
        .windows => "USERNAME",
        .macos => "USER",
        .linux => "USER",
        else => @compileError("current os not support"),
    };
    const user = env.get(key) orelse return error.UserNotFound;

    var rawConfig = try Config.init(alloc, io, user) orelse return;
    defer rawConfig.deinit();
    const config = rawConfig.value;

    for (config.sources) |source| {
        std.debug.print("from: {s}\nto: {s}\n", .{ source.from, source.to });
    }

    if (config.pause_after_finish) try waitForEnter(io);
}

fn waitForEnter(io: Io) !void {
    std.debug.print("按下回车键继续...", .{});
    const stdin = Io.File.stdin();
    var buffer: [4 * 1024]u8 = undefined;
    var stdin_reader = stdin.reader(io, &buffer);
    const reader = &stdin_reader.interface;
    _ = try reader.discardDelimiterExclusive('\n');
}
