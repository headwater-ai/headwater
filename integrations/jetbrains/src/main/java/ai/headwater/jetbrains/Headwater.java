// SPDX-License-Identifier: Apache-2.0
//
// Glue between the IntelliJ Platform and `Client`. It finds the project root,
// the path of a file inside it and the binary to run, and it opens a document.
// It decides nothing about what to ask the server or what counts as an answer:
// `Client` does, where `test/RunTests.java` holds it.

package ai.headwater.jetbrains;

import ai.headwater.jetbrains.Client.Options;
import ai.headwater.jetbrains.Client.Pointer;
import com.intellij.openapi.fileEditor.FileEditorManager;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.project.ProjectUtil;
import com.intellij.openapi.vfs.VfsUtilCore;
import com.intellij.openapi.vfs.VirtualFile;
import java.nio.file.Path;

final class Headwater {
    private Headwater() {}

    /** The environment variable that names the binary; `headwater` on the path otherwise. */
    static final String BIN = "HEADWATER_BIN";

    static VirtualFile root(Project project) {
        if (project == null || project.isDisposed()) return null;
        VirtualFile dir = ProjectUtil.guessProjectDir(project);
        return dir != null && dir.isInLocalFileSystem() ? dir : null;
    }

    /** How to reach the engine for this project, or null when the project has no local root. */
    static Options options(Project project) {
        VirtualFile root = root(project);
        if (root == null) return null;
        String bin = System.getenv(BIN);
        return Options.of(Path.of(root.getPath()), bin == null || bin.isEmpty() ? "headwater" : bin);
    }

    /** The path of `file` relative to the project root, or null for a file outside it. */
    static String relative(Project project, VirtualFile file) {
        VirtualFile root = root(project);
        if (root == null || file == null || !file.isInLocalFileSystem() || file.isDirectory()) return null;
        String relative = VfsUtilCore.getRelativePath(file, root, '/');
        return relative == null || relative.isEmpty() ? null : relative;
    }

    /** Opens the document a pointer names, when it exists under the project root. */
    static void open(Project project, Pointer pointer) {
        VirtualFile root = root(project);
        if (root == null) return;
        VirtualFile target = root.findFileByRelativePath(pointer.path());
        if (target != null) FileEditorManager.getInstance(project).openFile(target, true);
    }
}
