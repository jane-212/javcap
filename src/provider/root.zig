const std = @import("std");
const Allocator = std.mem.Allocator;
const Io = std.Io;
const domain = @import("domain");
const Javmenu = @import("Javmenu.zig");
const Avsox = @import("Avsox.zig");

pub fn all(alloc: Allocator, io: Io) ![]Provider {
    var providers: std.ArrayList(Provider) = .empty;
    defer providers.deinit(alloc);

    const javmenu = try Javmenu.init(alloc, io);
    try providers.append(alloc, javmenu.asProvider());

    const avsox = try Avsox.init(alloc, io);
    try providers.append(alloc, avsox.asProvider());

    const ownedProviders = try providers.toOwnedSlice(alloc);
    errdefer {
        for (ownedProviders) |*p| p.deinit();
        alloc.free(ownedProviders);
    }

    return ownedProviders;
}

pub const Provider = struct {
    ptr: *anyopaque,
    vtable: *const VTable,

    pub fn search(
        self: Provider,
        alloc: Allocator,
        key: domain.jav.Key,
    ) !domain.Nfo {
        return self.vtable.search(self.ptr, alloc, key);
    }

    pub fn name(
        self: Provider,
    ) []const u8 {
        return self.vtable.name(self.ptr);
    }

    pub fn deinit(self: Provider) void {
        self.vtable.deinit(self.ptr);
    }
};

pub const VTable = struct {
    search: *const fn (
        *anyopaque,
        Allocator,
        domain.jav.Key,
    ) anyerror!domain.Nfo,

    name: *const fn (
        *anyopaque,
    ) []const u8,

    deinit: *const fn (*anyopaque) void,
};

test {
    _ = @import("Javmenu.zig");
    _ = @import("Avsox.zig");
}
