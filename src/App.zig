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
    self.* = undefined;
}

pub fn start(self: *Self) !void {
    const rootProgress = std.Progress.start(self.io, .{
        .estimated_total_items = self.config.sources.len,
        .root_name = ".",
    });
    defer rootProgress.end();

    var group: Io.Group = .init;
    defer group.cancel(self.io);

    for (self.config.sources) |source| {
        try group.concurrent(self.io, Self.run, .{ self, rootProgress, source });
    }

    try group.await(self.io);

    if (self.config.pause_after_finish) try self.waitForEnter();
}

fn run(self: *Self, rootProgress: std.Progress.Node, source: Config.Source) void {
    self.runInner(rootProgress, source) catch |err| switch (err) {
        else => {},
    };
    rootProgress.completeOne();
}

fn runInner(self: *Self, rootProgress: std.Progress.Node, source: Config.Source) !void {
    const backend = try storage.load(self.alloc, self.io, source.type);
    defer backend.deinit();

    var manager = try self.loadSource(self.alloc, source, backend);
    defer manager.deinit(self.alloc);

    var group: Io.Group = .init;
    defer group.cancel(self.io);

    const taskProgress = rootProgress.start(source.from, manager.tasks.len);
    defer taskProgress.end();

    for (manager.tasks) |task| {
        try self.semaphore.wait(self.io);
        try group.concurrent(self.io, Self.runTask, .{ self, taskProgress, backend, task });
    }

    try group.await(self.io);
}

pub fn runTask(self: *Self, taskProgress: std.Progress.Node, backend: storage.Storage, task: Task) void {
    defer self.semaphore.post(self.io);

    self.runTaskInner(taskProgress, backend, task) catch |err| switch (err) {
        else => {},
    };
    taskProgress.completeOne();
}

fn runTaskInner(self: *Self, taskProgress: std.Progress.Node, backend: storage.Storage, task: Task) !void {
    var arena: std.heap.ArenaAllocator = .init(self.alloc);
    defer arena.deinit();
    const alloc = arena.allocator();

    var nfo = try domain.Nfo.init(alloc);
    defer nfo.deinit();

    const show = try task.file.key.show(alloc);
    defer alloc.free(show);

    const providerProgress = taskProgress.start(show, self.providers.len);
    defer providerProgress.end();

    for (self.providers) |p| {
        const c = providerProgress.start(p.name(), 0);
        defer c.end();

        var n = try p.search(alloc, task.file.key);
        defer n.deinit();

        try nfo.merge(&n);
        providerProgress.completeOne();
    }

    _ = try writeTo(alloc, backend, &task, &nfo);
}

fn writeTo(alloc: Allocator, backend: storage.Storage, task: *const Task, nfo: *const domain.Nfo) !bool {
    const show = try task.file.key.show(alloc);
    defer alloc.free(show);
    const to = task.to;

    const status = try backend.createDir(to);
    switch (status) {
        .created => {},
        .existed => return true,
    }

    if (nfo.poster) |p| try writeFile(.poster, alloc, backend, show, to, p);
    if (nfo.fanart) |f| try writeFile(.fanart, alloc, backend, show, to, f);
    if (nfo.subtitle) |s| try writeFile(.subtitle, alloc, backend, show, to, s);

    var buffer = Io.Writer.Allocating.init(alloc);
    defer buffer.deinit();
    const w = &buffer.writer;
    try writer.format(.xml, w, nfo);
    const n = buffer.written();
    try writeFile(.nfo, alloc, backend, show, to, n);

    const mediaToName = try std.fmt.allocPrint(alloc, "{s}{s}", .{
        show,
        task.file.ext,
    });
    defer alloc.free(mediaToName);

    const mediaTo = try std.fs.path.join(alloc, &.{ to, mediaToName });
    defer alloc.free(mediaTo);

    try backend.rename(
        task.path,
        mediaTo,
    );

    return false;
}

const WriteType = enum {
    poster,
    fanart,
    nfo,
    subtitle,
};

fn writeFile(comptime t: WriteType, alloc: Allocator, backend: storage.Storage, show: []const u8, to: []const u8, content: []const u8) !void {
    const fileName = try getFileName(t, alloc, show);
    defer alloc.free(fileName);

    const path = try std.fs.path.join(alloc, &.{ to, fileName });
    defer alloc.free(path);

    try backend.write(path, content);
}

fn getFileName(comptime t: WriteType, alloc: Allocator, show: []const u8) ![]const u8 {
    switch (t) {
        .poster => return std.fmt.allocPrint(alloc, "{s}-poster.jpg", .{show}),
        .fanart => return std.fmt.allocPrint(alloc, "{s}-fanart.jpg", .{show}),
        .nfo => return std.fmt.allocPrint(alloc, "{s}.nfo", .{show}),
        .subtitle => return std.fmt.allocPrint(alloc, "{s}.srt", .{show}),
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
    tasks: []Task,

    pub fn deinit(self: *Manager, alloc: Allocator) void {
        for (self.tasks) |*t| t.deinit(alloc);
        alloc.free(self.tasks);
        self.* = undefined;
    }
};
