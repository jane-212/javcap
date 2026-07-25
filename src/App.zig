const std = @import("std");
const Config = @import("Config.zig");
const Io = std.Io;

const Self = @This();

io: Io,
config: Config,

pub fn init(io: Io, config: Config) Self {
    return .{
        .io = io,
        .config = config,
    };
}

pub fn start(self: *Self) !void {
    for (self.config.sources) |source| std.debug.print("{s}\n", .{source.from});

    if (self.config.pause_after_finish) try self.waitForEnter();
}

fn waitForEnter(self: *const Self) !void {
    std.debug.print("按下回车键继续...", .{});
    var buffer: [4096]u8 = undefined;
    const stdin = Io.File.stdin();
    var stdin_reader = stdin.reader(self.io, &buffer);
    const reader = &stdin_reader.interface;
    _ = try reader.discardDelimiterExclusive('\n');
}
