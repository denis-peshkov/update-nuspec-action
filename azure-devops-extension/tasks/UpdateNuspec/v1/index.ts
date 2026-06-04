import * as path from 'path';
import * as tl from 'azure-pipelines-task-lib/task';

async function run(): Promise<void> {
    try {
        const dirInput = tl.getPathInput('dir', false, false);
        const dir = dirInput && dirInput.length > 0
            ? dirInput
            : (process.env['BUILD_SOURCESDIRECTORY'] ?? process.cwd());
        const dryRun = tl.getBoolInput('dryRun', false);

        const rid = process.platform === 'win32' ? 'win-x64' : 'linux-x64';
        const exeName = process.platform === 'win32' ? 'UpdateNuspecTool.exe' : 'UpdateNuspecTool';
        const exePath = path.join(__dirname, rid, exeName);

        if (!tl.exist(exePath)) {
            tl.setResult(
                tl.TaskResult.Failed,
                `UpdateNuspecTool binary not found at ${exePath}. Supported agents: windows-latest, ubuntu-latest.`);
            return;
        }

        const tool = tl.tool(exePath);
        tool.arg(dir);
        if (dryRun) {
            tool.arg('--dry-run');
        }

        const exitCode = await tool.exec();
        if (exitCode !== 0) {
            tl.setResult(tl.TaskResult.Failed, `UpdateNuspecTool exited with code ${exitCode}`);
        }
    }
    catch (err) {
        tl.setResult(tl.TaskResult.Failed, (err as Error).message);
    }
}

void run();
