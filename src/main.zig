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
}
