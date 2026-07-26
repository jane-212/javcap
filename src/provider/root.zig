const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const domain = @import("domain");

pub fn all(alloc: Allocator, io: Io) ![]Provider {
    _ = alloc;
    _ = io;

    return &.{};
}

pub const Provider = struct {
    ptr: *anyopaque,
    vtable: *const VTable,

    pub fn search(
        self: Provider,
        alloc: Allocator,
        path: []const u8,
        exts: []const []const u8,
    ) !domain.Nfo {
        return self.vtable.scan(self.ptr, alloc, path, exts);
    }

    pub fn deinit(self: Provider) void {
        self.vtable.deinit(self.ptr);
    }
};

pub const VTable = struct {
    scan: *const fn (
        *anyopaque,
        Allocator,
        []const u8,
        []const []const u8,
    ) anyerror!domain.Nfo,

    deinit: *const fn (*anyopaque) void,
};
