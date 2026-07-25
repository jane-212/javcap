pub const Config = @import("Config.zig");
pub const App = @import("App.zig");

test {
    _ = @import("Config.zig");
    _ = @import("App.zig");
    _ = @import("domain");
    _ = @import("media");
}
