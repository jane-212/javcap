const std = @import("std");
const Config = @import("Config.zig");
const Io = std.Io;
const media = @import("media");
const Allocator = std.mem.Allocator;

const Self = @This();

alloc: Allocator,
io: Io,
config: *const Config,

pub fn init(alloc: Allocator, io: Io, config: *const Config) Self {
    return .{
        .alloc = alloc,
        .io = io,
        .config = config,
    };
}

pub fn start(self: *const Self) !void {
    for (self.config.sources) |source| {
        const scanner = try media.scanner.load(self.alloc, self.io, source.type);
        defer scanner.deinit();

        const entries = try scanner.scan(self.alloc, source.from, source.exts);
        defer {
            for (entries) |*e| e.deinit(self.alloc);
            self.alloc.free(entries);
        }

        for (entries) |entry| {
            std.debug.print("{s}\n", .{entry.path});
        }
    }

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
