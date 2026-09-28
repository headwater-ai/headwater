// SPDX-License-Identifier: Apache-2.0
//
// Write time: when a file under a project is saved, a balloon names the
// documents that govern it. The listener only queues `Client.governing` on a
// pooled thread and returns, so the save never waits on the engine, and the
// balloon comes after the save. No pointers means no balloon.

package ai.headwater.jetbrains;

import ai.headwater.jetbrains.Client.Options;
import ai.headwater.jetbrains.Client.Pointer;
import com.intellij.notification.NotificationGroupManager;
import com.intellij.notification.NotificationType;
import com.intellij.openapi.application.ApplicationManager;
import com.intellij.openapi.editor.Document;
import com.intellij.openapi.fileEditor.FileDocumentManager;
import com.intellij.openapi.fileEditor.FileDocumentManagerListener;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.project.ProjectLocator;
import com.intellij.openapi.util.text.StringUtil;
import com.intellij.openapi.vfs.VirtualFile;
import java.util.List;
import java.util.stream.Collectors;
import org.jetbrains.annotations.NotNull;

public final class SaveListener implements FileDocumentManagerListener {
    /** The notification group `plugin.xml` declares. */
    static final String GROUP = "Headwater";

    @Override
    public void beforeDocumentSaving(@NotNull Document document) {
        VirtualFile file = FileDocumentManager.getInstance().getFile(document);
        if (file == null) return;
        Project project = ProjectLocator.getInstance().guessProjectForFile(file);
        String relative = Headwater.relative(project, file);
        Options options = Headwater.options(project);
        if (relative == null || options == null) return;
        ApplicationManager.getApplication().executeOnPooledThread(() -> {
            List<Pointer> pointers = Client.governing(relative, options);
            if (pointers.isEmpty() || project.isDisposed()) return;
            String names = pointers.stream().map(p -> StringUtil.escapeXmlEntities(Client.label(p))).collect(Collectors.joining("<br>"));
            NotificationGroupManager.getInstance()
                    .getNotificationGroup(GROUP)
                    .createNotification("Headwater: " + StringUtil.escapeXmlEntities(relative) + " is governed by", names, NotificationType.INFORMATION)
                    .notify(project);
        });
    }
}
