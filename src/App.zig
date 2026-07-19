const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const domain = @import("domain");

const Self = @This();

io: Io,
alloc: Allocator,
entries: std.ArrayList(domain.Entry),

pub fn init(alloc: Allocator, io: Io) !Self {
    return .{
        .io = io,
        .alloc = alloc,
        .entries = try .initCapacity(alloc, 8),
    };
}

pub fn addSource(self: *Self, source: domain.Source) !void {
    var arena = std.heap.ArenaAllocator.init(self.alloc);
    defer arena.deinit();

    const alloc = arena.allocator();

    const cwd = Io.Dir.cwd();
    const dir = try cwd.openDir(self.io, source.from, .{});
    defer dir.close(self.io);

    var walker = try dir.walk(alloc);
    defer walker.deinit();

    while (try walker.next(self.io)) |entry| {
        if (entry.kind != .file) continue;

        const file = entry.basename;
        const dot_ext = std.fs.path.extension(file);
        const ext = std.mem.trimStart(u8, dot_ext, ".");
        const upper = try std.ascii.allocUpperString(alloc, ext);
        defer alloc.free(upper);
        if (!try isVideo(alloc, upper, source.exts)) continue;
        const name = std.fs.path.stem(file);
        const full_path = try std.fs.path.join(alloc, &.{ source.from, entry.path });
        defer alloc.free(full_path);

        try self.entries.append(self.alloc, try .init(self.alloc, source.type, name, file, source.to, full_path));
    }
}

pub fn start(self: *Self) !void {
    std.debug.print("*************************\n", .{});
    for (self.entries.items) |entry| std.debug.print("key: {s}\ntype: {}\nfile: {s}\npath: {s}\ndest: {s}\n*************************\n", .{ entry.key, entry.type, entry.file, entry.path, entry.dest });
}

fn isVideo(alloc: Allocator, ext: []const u8, exts: [][]const u8) !bool {
    for (exts) |e| {
        const upper = try std.ascii.allocUpperString(alloc, e);
        defer alloc.free(upper);
        if (std.mem.eql(u8, upper, ext)) return true;
    }
    return false;
}

pub fn deinit(self: *Self) void {
    for (self.entries.items) |entry| entry.deinit();
    self.entries.deinit(self.alloc);
}
