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
            return try s.write(path, content);
        }

        fn deinitInner(
            ptr: *anyopaque,
        ) void {
            const s: *Self = @ptrCast(@alignCast(ptr));
            s.deinit();
        }

        const vtable = storage.VTable{
            .write = writeInner,
            .deinit = deinitInner,
        };
    };

    return .{
        .ptr = self,
        .vtable = &Impl.vtable,
    };
}

pub fn write(self: *const Self, path: []const u8, content: []const u8) !void {
    _ = self;
    _ = path;
    _ = content;
}
