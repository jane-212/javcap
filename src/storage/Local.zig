const std = @import("std");
const storage = @import("root.zig");
const Allocator = std.mem.Allocator;
const Io = std.Io;

const Self = @This();

alloc: Allocator,
io: Io,

pub fn init(alloc: Allocator, io: Io) !*Self {
    const self = try alloc.create(Self);
    errdefer alloc.destroy(self);

    self.* = .{
        .alloc = alloc,
        .io = io,
    };

    return self;
}

pub fn deinit(self: *Self) void {
    self.alloc.destroy(self);
    self.* = undefined;
}

pub fn asStorage(self: *Self) storage.Storage {
    const Impl = struct {
        fn writeInner(
            ptr: *anyopaque,
            path: []const u8,
            content: []const u8,
        ) !void {
            const s: *Self = @ptrCast(@alignCast(ptr));
            return s.write(path, content);
        }

        fn walkInner(
            ptr: *anyopaque,
            alloc: Allocator,
            path: []const u8,
        ) !storage.Walker {
            const s: *Self = @ptrCast(@alignCast(ptr));
            return s.walk(alloc, path);
        }

        fn listInner(
            ptr: *anyopaque,
            alloc: Allocator,
            path: []const u8,
        ) ![]storage.Entry {
            const s: *Self = @ptrCast(@alignCast(ptr));
            return s.list(alloc, path);
        }

        fn statsInner(
            ptr: *anyopaque,
            path: []const u8,
        ) !storage.FileType {
            const s: *Self = @ptrCast(@alignCast(ptr));
            return s.stats(path);
        }

        fn renameInner(
            ptr: *anyopaque,
            old: []const u8,
            new: []const u8,
        ) !void {
            const s: *Self = @ptrCast(@alignCast(ptr));
            return s.rename(old, new);
        }

        fn createDirInner(
            ptr: *anyopaque,
            path: []const u8,
        ) !Io.Dir.CreatePathStatus {
            const s: *Self = @ptrCast(@alignCast(ptr));
            return s.createDir(path);
        }

        fn deinitInner(
            ptr: *anyopaque,
        ) void {
            const s: *Self = @ptrCast(@alignCast(ptr));
            s.deinit();
        }

        const vtable = storage.VTable{
            .write = writeInner,
            .walk = walkInner,
            .list = listInner,
            .stats = statsInner,
            .rename = renameInner,
            .createDir = createDirInner,
            .deinit = deinitInner,
        };
    };

    return .{
        .ptr = self,
        .vtable = &Impl.vtable,
    };
}

pub fn write(self: *Self, path: []const u8, content: []const u8) !void {
    const file = Io.Dir.openFileAbsolute(self.io, path, .{ .mode = .write_only }) catch |err| switch (err) {
        error.FileNotFound => try Io.Dir.createFileAbsolute(self.io, path, .{}),
        else => return err,
    };
    defer file.close(self.io);

    var buffer: [4096]u8 = undefined;
    var writer = file.writer(self.io, &buffer);

    const w = &writer.interface;

    try w.writeAll(content);
    try w.flush();
}

pub fn walk(self: *Self, alloc: Allocator, path: []const u8) !storage.Walker {
    var walker = try storage.Walker.init(alloc, self.asStorage());
    errdefer walker.deinit();

    const p = try alloc.dupe(u8, path);
    errdefer alloc.free(p);

    try walker.push(.{
        .fileType = try self.stats(path),
        .path = p,
    });

    return walker;
}

pub fn list(self: *Self, alloc: Allocator, path: []const u8) ![]storage.Entry {
    const root = try Io.Dir.openDirAbsolute(self.io, path, .{});
    defer root.close(self.io);

    var entries = try std.ArrayList(storage.Entry).initCapacity(self.alloc, 8);
    errdefer for (entries.items) |*e| e.deinit(alloc);
    defer entries.deinit(self.alloc);

    var it = root.iterate();
    while (try it.next(self.io)) |entry| {
        const p = try std.fs.path.join(alloc, &.{ path, entry.name });
        errdefer alloc.free(p);

        try entries.append(self.alloc, .{
            .fileType = try self.stats(p),
            .path = p,
        });
    }

    return entries.toOwnedSlice(alloc);
}

pub fn stats(self: *Self, path: []const u8) !storage.FileType {
    const file = try Io.Dir.openFileAbsolute(self.io, path, .{});
    defer file.close(self.io);

    const stat = try file.stat(self.io);
    switch (stat.kind) {
        .file => return .file,
        .directory => return .dir,
        else => return .other,
    }
}

pub fn rename(self: *Self, old: []const u8, new: []const u8) !void {
    try Io.Dir.renameAbsolute(old, new, self.io);
}

pub fn createDir(self: *Self, path: []const u8) !Io.Dir.CreatePathStatus {
    return Io.Dir.createDirPathStatus(.cwd(), self.io, path);
}
