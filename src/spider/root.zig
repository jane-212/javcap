pub const Spider = @import("Spider.zig");
const domain = @import("domain");

pub const Crawler = struct {
    ptr: *anyopaque,
    vtable: *const VTable,

    const Self = @This();
    const Data = domain.Nfo;

    pub const VTable = struct {
        search: *const fn (ptr: *anyopaque, key: []const u8) anyerror!Data,
        deinit: *const fn (ptr: *anyopaque) void,
    };

    pub fn search(self: Self, key: []const u8) !Data {
        self.vtable.search(self.ptr, key);
    }

    pub fn deinit(self: Self) void {
        self.vtable.deinit(self.ptr);
    }

    fn asSpider(impl: anytype) Self {
        const T = @TypeOf(impl);
        const vtable = struct {
            fn search(ptr: *anyopaque, key: []const u8) !Data {
                const self: *T = @ptrCast(@alignCast(ptr));
                self.search(key);
            }

            fn deinit(ptr: *anyopaque) void {
                const self: *T = @ptrCast(@alignCast(ptr));
                self.deinit();
            }
        };

        return .{
            .ptr = impl,
            .vtable = &.{
                .search = vtable.search,
                .deinit = vtable.deinit,
            },
        };
    }
};
