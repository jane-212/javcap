const std = @import("std");
const Config = @import("Config.zig");
const Io = std.Io;
const media = @import("media");
const Allocator = std.mem.Allocator;
const domain = @import("domain");

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

pub fn start(self: *Self) !void {
    const tasks = try self.loadAllSources(self.alloc);
    defer {
        for (tasks) |*t| t.deinit();
        self.alloc.free(tasks);
    }

    std.debug.print("**************************\n", .{});
    for (tasks) |task| {
        std.debug.print("type: {}\n", .{task.type});
        std.debug.print("path: {s}\n", .{task.path});
        std.debug.print("file name: {s}\n", .{task.file.name});
        switch (task.file.key) {
            .jav => |jav| std.debug.print("file key: {s}-{s}\n", .{ jav.id, jav.number }),
            .fc2 => |fc2| std.debug.print("file key: FC2-{s}\n", .{fc2}),
            .normal => |normal| std.debug.print("{s}\n", .{normal}),
        }
        std.debug.print("**************************\n", .{});
    }

    if (self.config.pause_after_finish) try self.waitForEnter();
}

fn loadAllSources(self: *Self, alloc: Allocator) ![]Task {
    var tasks: std.ArrayList(Task) = .empty;
    for (self.config.sources) |source| {
        const scanner = try media.scanner.load(self.alloc, self.io, source.type);
        defer scanner.deinit();

        const entries = try scanner.scan(self.alloc, source.from, source.exts);
        defer {
            for (entries) |*e| e.deinit(self.alloc);
            self.alloc.free(entries);
        }

        for (entries) |entry| {
            const parsedFile = try media.FileParser.parse(alloc, entry.path);

            try tasks.append(alloc, .{
                .alloc = alloc,
                .type = entry.type,
                .file = parsedFile,
                .path = try alloc.dupe(u8, entry.path),
            });
        }
    }
    return tasks.toOwnedSlice(alloc);
}

fn waitForEnter(self: *const Self) !void {
    std.debug.print("按下回车键继续...", .{});
    var buffer: [4096]u8 = undefined;
    const stdin = Io.File.stdin();
    var stdin_reader = stdin.reader(self.io, &buffer);
    const reader = &stdin_reader.interface;
    _ = try reader.discardDelimiterExclusive('\n');
}

const Task = struct {
    alloc: Allocator,
    type: domain.storage.Type,
    file: media.FileParser.ParsedFile,
    path: []const u8,

    pub fn deinit(self: *Task) void {
        self.alloc.free(self.path);
        self.file.deinit();
        self.* = undefined;
    }
};
