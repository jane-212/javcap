const std = @import("std");
const Config = @import("Config.zig");
const Io = std.Io;
const media = @import("media");
const Allocator = std.mem.Allocator;
const domain = @import("domain");
const provider = @import("provider");
const writer = @import("writer");
const builtin = @import("builtin");
const storage = @import("storage");

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
    for (self.config.sources) |source| {
        try self.run(source);
    }

    if (self.config.pause_after_finish) try self.waitForEnter();
}

fn run(self: *Self, source: Config.Source) !void {
    const backend = try storage.load(self.alloc, self.io, source.type);
    defer backend.deinit();

    var manager = try self.loadSource(self.alloc, source, backend);
    defer manager.deinit(self.alloc);

    var group: Io.Group = .init;
    defer group.cancel(self.io);

    for (manager.tasks) |task| {
        try self.semaphore.wait(self.io);
        try group.concurrent(self.io, Self.runTask, .{ self, task });
    }

    try group.await(self.io);
}

pub fn runTask(self: *Self, task: Task) void {
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
            try w.interface.writeAll("*********************\n");
            try writer.format(.normal, &w.interface, &n);
            try w.interface.writeAll("*********************\n");
            try w.flush();
        }

        try nfo.merge(&n);
    }

    if (builtin.mode == .Debug) {
        var buffer: [4096]u8 = undefined;
        var w = Io.File.stderr().writer(self.io, &buffer);
        try w.interface.writeAll("#####################\n");
        try writer.format(.normal, &w.interface, &nfo);
        try w.interface.writeAll("#####################\n");
        try w.flush();
    }
}

fn loadSource(self: *Self, alloc: Allocator, source: Config.Source, backend: storage.Storage) !Manager {
    var tasks: std.ArrayList(Task) = .empty;
    defer tasks.deinit(self.alloc);
    errdefer for (tasks.items) |*t| t.deinit(alloc);

    const scanner = try media.Scanner.init(self.alloc, self.io, backend);

    const entries = try scanner.scan(self.alloc, source.from, source.exts);
    defer {
        for (entries) |*e| e.deinit(self.alloc);
        self.alloc.free(entries);
    }

    for (entries) |entry| {
        var parsedFile = try media.FileParser.parse(alloc, entry.path);
        errdefer parsedFile.deinit();
        const path = try alloc.dupe(u8, entry.path);
        errdefer alloc.free(path);
        const to = try alloc.dupe(u8, source.to);
        errdefer alloc.free(to);

        try tasks.append(self.alloc, .{
            .file = parsedFile,
            .path = path,
            .to = to,
        });
    }

    return .{
        .backend = backend,
        .tasks = try tasks.toOwnedSlice(alloc),
    };
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
    file: media.FileParser.ParsedFile,
    path: []const u8,
    to: []const u8,

    pub fn deinit(self: *Task, alloc: Allocator) void {
        alloc.free(self.path);
        alloc.free(self.to);
        self.file.deinit();
        self.* = undefined;
    }
};

const Manager = struct {
    backend: storage.Storage,
    tasks: []Task,

    pub fn deinit(self: *Manager, alloc: Allocator) void {
        for (self.tasks) |*t| t.deinit(alloc);
        alloc.free(self.tasks);
        self.backend.deinit();
        self.* = undefined;
    }
};
