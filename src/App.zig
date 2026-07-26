const std = @import("std");
const Config = @import("Config.zig");
const Io = std.Io;
const media = @import("media");
const Allocator = std.mem.Allocator;
const domain = @import("domain");
const provider = @import("provider");
const writer = @import("writer");
const builtin = @import("builtin");

const Self = @This();

alloc: Allocator,
io: Io,
config: *const Config,
semaphore: Io.Semaphore,
providers: []provider.Provider,

pub fn init(alloc: Allocator, io: Io, config: *const Config) !Self {
    const providers = try provider.all(alloc, io);
    errdefer {
        for (providers) |*p| p.deinit();
        alloc.free(providers);
    }

    return .{
        .alloc = alloc,
        .io = io,
        .config = config,
        .semaphore = .{ .permits = 5 },
        .providers = providers,
    };
}

pub fn deinit(self: *Self) void {
    for (self.providers) |*p| p.deinit();
    self.alloc.free(self.providers);
}

pub fn start(self: *Self) !void {
    const tasks = try self.loadAllSources(self.alloc);
    defer {
        for (tasks) |*t| t.deinit();
        self.alloc.free(tasks);
    }

    var group: Io.Group = .init;
    defer group.cancel(self.io);

    for (tasks) |task| {
        try self.semaphore.wait(self.io);

        try group.concurrent(self.io, Self.runTask, .{ self, task });
    }

    try group.await(self.io);

    if (self.config.pause_after_finish) try self.waitForEnter();
}

fn runTask(self: *Self, task: Task) void {
    defer self.semaphore.post(self.io);
    self.runTaskInner(task) catch |err| switch (err) {
        else => {},
    };
}

fn runTaskInner(self: *Self, task: Task) !void {
    var arena: std.heap.ArenaAllocator = .init(self.alloc);
    defer arena.deinit();
    const alloc = arena.allocator();

    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();

    for (self.providers) |p| {
        var n = try p.search(alloc, task.file.key);
        defer n.deinit();

        if (builtin.mode == .Debug) {
            var buffer: [4096]u8 = undefined;
            var w = Io.File.stderr().writer(self.io, &buffer);
            try writer.format(.normal, &w.interface, &n);
            try w.flush();
        }

        try nfo.merge(&n);
    }

    if (builtin.mode == .Debug) {
        var buffer: [4096]u8 = undefined;
        var w = Io.File.stderr().writer(self.io, &buffer);
        try writer.format(.normal, &w.interface, &nfo);
        try w.flush();
    }
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
            const path = try alloc.dupe(u8, entry.path);

            try tasks.append(alloc, .{
                .alloc = alloc,
                .type = entry.type,
                .file = parsedFile,
                .path = path,
            });
        }
    }
    const ownedTasks = try tasks.toOwnedSlice(alloc);
    errdefer {
        for (ownedTasks) |*t| t.deinit();
        alloc.free(ownedTasks);
    }

    return ownedTasks;
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
