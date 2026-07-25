const std = @import("std");
const javcap = @import("javcap");
const Config = javcap.Config;
const Io = std.Io;

pub const known_folders_config = Config.known_folders_config;

pub fn main(init: std.process.Init) !void {
    const alloc = init.gpa;
    const io = init.io;
    const env = init.environ_map;

    var context: Config.Context = .{};
    defer context.deinit(alloc);
    var configRaw = Config.init(alloc, io, &context, env) catch |err| switch (err) {
        error.ValidateFailed => {
            var buffer: [4096]u8 = undefined;
            var stderr = std.Io.File.stderr().writer(io, &buffer);
            try context.formatValidate(&stderr.interface);
            return;
        },
        error.ParseZon => {
            var buffer: [4096]u8 = undefined;
            var stderr = std.Io.File.stderr().writer(io, &buffer);
            try context.formatDiagnostics(&stderr.interface);
            return;
        },
        else => return err,
    };
    defer configRaw.deinit();

    const config = configRaw.value;
    for (config.sources) |source| std.debug.print("{s}\n", .{source.from});

    if (config.pause_after_finish) try waitForEnter(io);
}

fn waitForEnter(io: Io) !void {
    std.debug.print("按下回车键继续...", .{});
    var buffer: [4096]u8 = undefined;
    const stdin = Io.File.stdin();
    var stdin_reader = stdin.reader(io, &buffer);
    const reader = &stdin_reader.interface;
    _ = try reader.discardDelimiterExclusive('\n');
}
