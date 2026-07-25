const std = @import("std");
const scanner = @import("root.zig");
const Allocator = std.mem.Allocator;
const Io = std.Io;

const Self = @This();

alloc: Allocator,
io: Io,

pub fn init(alloc: Allocator, io: Io) !*Self {
    const self = try alloc.create(Self);
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

pub fn asScanner(self: *Self) scanner.Scanner {
    const Impl = struct {
        fn scanInner(
            ptr: *anyopaque,
            alloc: Allocator,
            path: []const u8,
            exts: []const []const u8,
        ) ![]scanner.Entry {
            const s: *Self = @ptrCast(@alignCast(ptr));
            return try s.scan(alloc, path, exts);
        }

        fn deinitInner(
            ptr: *anyopaque,
        ) void {
            const s: *Self = @ptrCast(@alignCast(ptr));
            s.deinit();
        }

        const vtable = scanner.VTable{
            .scan = scanInner,
            .deinit = deinitInner,
        };
    };

    return .{
        .ptr = self,
        .vtable = &Impl.vtable,
    };
}

pub fn scan(self: *const Self, alloc: Allocator, path: []const u8, exts: []const []const u8) ![]scanner.Entry {
    const cwd = try Io.Dir.openDirAbsolute(self.io, path, .{});
    defer cwd.close(self.io);

    var walker = try cwd.walk(self.alloc);
    defer walker.deinit();

    var entries: std.ArrayList(scanner.Entry) = .empty;
    defer entries.deinit(self.alloc);

    while (try walker.next(self.io)) |entry| {
        if (entry.kind != .file) continue;

        const p = entry.path;

        const extWithDot = std.fs.path.extension(p);
        const ext = if (extWithDot.len == 0) extWithDot else extWithDot[1..];
        if (!scanner.matchExts(ext, exts)) continue;

        try entries.append(self.alloc, .{
            .type = .local,
            .path = try alloc.dupe(u8, p),
        });
    }

    return entries.toOwnedSlice(alloc);
}
