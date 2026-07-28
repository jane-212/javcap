const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const domain = @import("domain");
const Local = @import("Local.zig");

pub fn load(alloc: Allocator, io: Io, t: domain.storage.Type) !Storage {
    switch (t) {
        .local => {
            var localStorage = try Local.init(alloc, io);
            return localStorage.asStorage();
        },
    }
}

pub const Storage = struct {
    ptr: *anyopaque,
    vtable: *const VTable,

    pub fn write(
        self: Storage,
        path: []const u8,
        content: []const u8,
    ) !void {
        return self.vtable.write(self.ptr, path, content);
    }

    pub fn walk(
        self: Storage,
        alloc: Allocator,
        path: []const u8,
    ) !Walker {
        return self.vtable.walk(self.ptr, alloc, path);
    }

    pub fn list(
        self: Storage,
        alloc: Allocator,
        path: []const u8,
    ) ![]Entry {
        return self.vtable.list(self.ptr, alloc, path);
    }

    pub fn stats(
        self: Storage,
        path: []const u8,
    ) !FileType {
        return self.vtable.stats(self.ptr, path);
    }

    pub fn rename(
        self: Storage,
        old: []const u8,
        new: []const u8,
    ) !void {
        return self.vtable.rename(self.ptr, old, new);
    }

    pub fn deinit(self: Storage) void {
        self.vtable.deinit(self.ptr);
    }
};

pub const VTable = struct {
    write: *const fn (
        *anyopaque,
        []const u8,
        []const u8,
    ) anyerror!void,

    walk: *const fn (
        *anyopaque,
        Allocator,
        []const u8,
    ) anyerror!Walker,

    list: *const fn (
        *anyopaque,
        Allocator,
        []const u8,
    ) anyerror![]Entry,

    stats: *const fn (
        *anyopaque,
        []const u8,
    ) anyerror!FileType,

    rename: *const fn (
        *anyopaque,
        []const u8,
        []const u8,
    ) anyerror!void,

    deinit: *const fn (*anyopaque) void,
};

pub const Walker = struct {
    alloc: Allocator,
    stack: std.ArrayList(Entry),
    storage: Storage,

    pub fn init(alloc: Allocator, storage: Storage) !Walker {
        return .{
            .alloc = alloc,
            .stack = try std.ArrayList(Entry).initCapacity(alloc, 8),
            .storage = storage,
        };
    }

    pub fn next(self: *Walker) !?Entry {
        while (self.stack.items.len > 0) {
            var top = self.stack.pop() orelse return null;
            errdefer top.deinit(self.alloc);

            if (top.fileType == .dir) {
                const children = try self.storage.list(self.alloc, top.path);
                errdefer for (children) |*c| c.deinit(self.alloc);
                defer self.alloc.free(children);

                for (children) |c| try self.stack.append(self.alloc, c);
            }

            return top;
        }

        return null;
    }

    pub fn deinit(self: *Walker) void {
        for (self.stack.items) |*e| e.deinit(self.alloc);
        self.stack.deinit(self.alloc);
        self.* = undefined;
    }
};

pub const Entry = struct {
    path: []const u8,
    fileType: FileType,

    pub fn deinit(self: *Entry, alloc: Allocator) void {
        alloc.free(self.path);
        self.* = undefined;
    }
};

pub const FileType = enum {
    file,
    dir,
    other,
};

test {
    _ = @import("Local.zig");
}
