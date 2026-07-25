const std = @import("std");
const Allocator = std.mem.Allocator;
const domain = @import("domain");
const LocalScanner = @import("LocalScanner.zig");
const Io = std.Io;

pub fn load(alloc: Allocator, io: Io, t: domain.storage.Type) !Scanner {
    return switch (t) {
        .local => {
            var localScanner = try LocalScanner.init(alloc, io);
            return localScanner.asScanner();
        },
    };
}

pub const Scanner = struct {
    ptr: *anyopaque,
    vtable: *const VTable,

    pub fn scan(
        self: Scanner,
        alloc: Allocator,
        path: []const u8,
        exts: []const []const u8,
    ) ![]Entry {
        return self.vtable.scan(self.ptr, alloc, path, exts);
    }

    pub fn deinit(self: Scanner) void {
        self.vtable.deinit(self.ptr);
    }
};

pub const VTable = struct {
    scan: *const fn (
        *anyopaque,
        Allocator,
        []const u8,
        []const []const u8,
    ) anyerror![]Entry,

    deinit: *const fn (*anyopaque) void,
};

pub const Entry = struct {
    type: domain.storage.Type,
    path: []const u8,

    pub fn deinit(self: *Entry, alloc: Allocator) void {
        alloc.free(self.path);
        self.* = undefined;
    }
};

test {
    _ = @import("LocalScanner.zig");
}
