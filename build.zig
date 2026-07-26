const std = @import("std");
const zon = @import("build.zig.zon");
const builtin = @import("builtin");

pub fn build(b: *std.Build) void {
    const target = b.standardTargetOptions(.{});
    const optimize = b.standardOptimizeOption(.{});
    const strip = if (builtin.mode == .Debug) false else true;

    const options = b.addOptions();
    options.addOption([]const u8, "name", "javcap");
    options.addOption([]const u8, "version", zon.version);

    const mecha = b.dependency("mecha", .{
        .target = target,
        .optimize = optimize,
    });

    const known_folders = b.dependency("known_folders", .{
        .target = target,
        .optimize = optimize,
    });

    const domain = b.createModule(.{
        .root_source_file = b.path("src/domain/root.zig"),
        .target = target,
        .optimize = optimize,
        .strip = strip,
    });
    const domain_tests = b.addTest(.{
        .root_module = domain,
    });

    const infra = b.createModule(.{
        .root_source_file = b.path("src/infra/root.zig"),
        .target = target,
        .optimize = optimize,
        .strip = strip,
    });
    const infra_tests = b.addTest(.{
        .root_module = infra,
    });

    const provider = b.createModule(.{
        .root_source_file = b.path("src/provider/root.zig"),
        .target = target,
        .optimize = optimize,
        .strip = strip,
    });
    provider.addImport("domain", domain);
    provider.addImport("infra", infra);
    const provider_tests = b.addTest(.{
        .root_module = provider,
    });

    const media = b.createModule(.{
        .root_source_file = b.path("src/media/root.zig"),
        .target = target,
        .optimize = optimize,
        .strip = strip,
    });
    media.addImport("domain", domain);
    media.addImport("mecha", mecha.module("mecha"));
    const media_tests = b.addTest(.{
        .root_module = media,
    });

    const mod = b.addModule("javcap", .{
        .root_source_file = b.path("src/root.zig"),
        .target = target,
        .optimize = optimize,
        .strip = strip,
    });
    mod.addOptions("options", options);
    mod.addImport("media", media);
    mod.addImport("domain", domain);
    mod.addImport("provider", provider);
    mod.addImport("known-folders", known_folders.module("known-folders"));
    const mod_tests = b.addTest(.{
        .root_module = mod,
    });

    const exe = b.addExecutable(.{
        .name = "javcap",
        .root_module = b.createModule(.{
            .root_source_file = b.path("src/main.zig"),
            .target = target,
            .optimize = optimize,
            .strip = strip,
        }),
    });
    b.installArtifact(exe);
    exe.root_module.addImport("javcap", mod);
    const exe_tests = b.addTest(.{
        .root_module = exe.root_module,
    });

    const run_step = b.step("run", "Run the app");
    const run_cmd = b.addRunArtifact(exe);
    run_step.dependOn(&run_cmd.step);
    run_cmd.step.dependOn(b.getInstallStep());
    if (b.args) |args| {
        run_cmd.addArgs(args);
    }

    const test_step = b.step("test", "Run tests");
    const run_mod_tests = b.addRunArtifact(mod_tests);
    const run_exe_tests = b.addRunArtifact(exe_tests);
    const run_media_tests = b.addRunArtifact(media_tests);
    const run_domain_tests = b.addRunArtifact(domain_tests);
    const run_infra_tests = b.addRunArtifact(infra_tests);
    const run_provider_tests = b.addRunArtifact(provider_tests);
    test_step.dependOn(&run_mod_tests.step);
    test_step.dependOn(&run_exe_tests.step);
    test_step.dependOn(&run_media_tests.step);
    test_step.dependOn(&run_domain_tests.step);
    test_step.dependOn(&run_infra_tests.step);
    test_step.dependOn(&run_provider_tests.step);

    const check_step = b.step("check", "Check compile");
    check_step.dependOn(&run_cmd.step);
}
