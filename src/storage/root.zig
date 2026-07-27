const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const domain = @import("domain");
const Local = @import("Local.zig");

pub fn load(alloc: Allocator, io: Io, t: domain.storage.Type) !Storage {
    return switch (t) {
        .local => {
            var localStorage = try Local.init(alloc, io);
            errdefer localStorage.deinit();
            return localStorage.asStorage();
        },
    };
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

    deinit: *const fn (*anyopaque) void,
};

test {
    _ = @import("Local.zig");
}
