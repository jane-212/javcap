const std = @import("std");
const Io = std.Io;
const builtin = @import("builtin");
const Config = @import("Config.zig");
const App = @import("App.zig");

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

    var rawConfig = Config.init(alloc, io, user) catch |err| switch (err) {
        error.GenerateDefaultConfig => return,
        error.NotAbsolutePath => {
            std.debug.print("配置文件中的路径必须是绝对路径\n", .{});
            return;
        },
        else => {
            std.debug.print("加载配置文件失败, 请检查文件格式\n", .{});
            return;
        },
    };
    defer rawConfig.deinit();
    const config = rawConfig.value;

    var app = try App.init(alloc, io);
    defer app.deinit();

    for (config.sources) |s| try app.addSource(s);

    try app.start();

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
