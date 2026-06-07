import * as fs from 'fs';
import * as path from 'path';
import * as tl from 'azure-pipelines-task-lib/task';

async function run(): Promise<void> {
    try {
        const dirInput = tl.getPathInput('dir', false, false);
        const dir = dirInput && dirInput.length > 0
            ? dirInput
            : (process.env['BUILD_SOURCESDIRECTORY'] ?? process.cwd());
        const dryRun = tl.getBoolInput('dryRun', false);
        const packageVersion = tl.getInput('packageVersion', false)?.trim() ?? '';
        const dependencyScope = tl.getInput('dependencyScope', false)?.trim() ?? '';

        const rid = process.platform === 'win32' ? 'win-x64' : 'linux-x64';
        const exeName = process.platform === 'win32' ? 'UpdateNuspecTool.exe' : 'UpdateNuspecTool';
        const exePath = path.join(__dirname, rid, exeName);

        const ridDir = path.join(__dirname, rid);
        if (!tl.exist(exePath)) {
            const hint = fs.existsSync(ridDir)
                ? `Contents of ${ridDir}: ${fs.readdirSync(ridDir).join(', ')}`
                : `Missing folder ${ridDir}. Reinstall the extension from a CI build that publishes linux-x64/win-x64 binaries.`;
            tl.setResult(
                tl.TaskResult.Failed,
                `UpdateNuspecTool binary not found at ${exePath}. ${hint} Agents: windows-latest, ubuntu-latest.`);
            return;
        }

        if (process.platform !== 'win32') {
            try {
                fs.chmodSync(exePath, 0o755);
            }
            catch {
                // VSIX extract may drop executable bit; ignore chmod errors
            }
        }

        const tool = tl.tool(exePath);
        tool.arg(dir);
        if (dryRun) {
            tool.arg('--dry-run');
        }

        if (packageVersion.length > 0) {
            tool.arg('--package-version');
            tool.arg(packageVersion);
        }

        if (dependencyScope.length > 0) {
            tool.arg('--dependency-scope');
            tool.arg(dependencyScope);
        }

        const env: NodeJS.ProcessEnv = { ...process.env };
        if (!env.CONSOLE_ANSI_COLOR) {
            env.CONSOLE_ANSI_COLOR = 'true';
        }

        if (packageVersion.length > 0) {
            env.PACKAGE_VERSION = packageVersion;
        }

        if (dependencyScope.length > 0) {
            env.DEPENDENCY_SCOPE = dependencyScope;
        }

        const exitCode = await tool.exec({ env });
        if (exitCode !== 0) {
            tl.setResult(tl.TaskResult.Failed, `UpdateNuspecTool exited with code ${exitCode}`);
            return;
        }

        if (packageVersion.length > 0) {
            tl.setVariable('PackageVersion', packageVersion);
        }
    }
    catch (err) {
        tl.setResult(tl.TaskResult.Failed, (err as Error).message);
    }
}

void run();
