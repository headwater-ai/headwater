// SPDX-License-Identifier: Apache-2.0
//
// Read time: the *Headwater* tool window lists the documents that govern the
// file in the selected editor, and is empty when none do. Each change of
// selection runs `Client.governing` on a pooled thread, never on the event
// dispatch thread, and an answer that arrives after a later selection is
// dropped. Double-click or Enter on a row opens the document.

package ai.headwater.jetbrains;

import ai.headwater.jetbrains.Client.Options;
import ai.headwater.jetbrains.Client.Pointer;
import com.intellij.openapi.application.ApplicationManager;
import com.intellij.openapi.fileEditor.FileEditorManager;
import com.intellij.openapi.fileEditor.FileEditorManagerEvent;
import com.intellij.openapi.fileEditor.FileEditorManagerListener;
import com.intellij.openapi.project.DumbAware;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.vfs.VirtualFile;
import com.intellij.openapi.wm.ToolWindow;
import com.intellij.openapi.wm.ToolWindowFactory;
import com.intellij.ui.DoubleClickListener;
import com.intellij.ui.components.JBList;
import com.intellij.ui.components.JBScrollPane;
import com.intellij.ui.content.Content;
import com.intellij.ui.content.ContentFactory;
import java.awt.event.KeyAdapter;
import java.awt.event.KeyEvent;
import java.awt.event.MouseEvent;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import javax.swing.DefaultListModel;
import org.jetbrains.annotations.NotNull;

public final class GoverningToolWindowFactory implements ToolWindowFactory, DumbAware {
    @Override
    public void createToolWindowContent(@NotNull Project project, @NotNull ToolWindow toolWindow) {
        DefaultListModel<Pointer> model = new DefaultListModel<>();
        JBList<Pointer> list = new JBList<>(model);
        list.setCellRenderer(new PointerRenderer());
        list.getEmptyText().setText("No document governs this file");
        new DoubleClickListener() {
            @Override
            protected boolean onDoubleClick(@NotNull MouseEvent event) {
                Pointer chosen = list.getSelectedValue();
                if (chosen != null) Headwater.open(project, chosen);
                return chosen != null;
            }
        }.installOn(list);
        list.addKeyListener(new KeyAdapter() {
            @Override
            public void keyPressed(KeyEvent event) {
                Pointer chosen = list.getSelectedValue();
                if (event.getKeyCode() == KeyEvent.VK_ENTER && chosen != null) Headwater.open(project, chosen);
            }
        });
        Content content = ContentFactory.getInstance().createContent(new JBScrollPane(list), "", false);
        toolWindow.getContentManager().addContent(content);

        AtomicInteger generation = new AtomicInteger();
        Refresh refresh = file -> {
            int mine = generation.incrementAndGet();
            String relative = Headwater.relative(project, file);
            Options options = Headwater.options(project);
            if (relative == null || options == null) {
                show(model, List.of());
                return;
            }
            ApplicationManager.getApplication().executeOnPooledThread(() -> {
                List<Pointer> pointers = Client.governing(relative, options);
                ApplicationManager.getApplication().invokeLater(() -> {
                    if (mine == generation.get()) show(model, pointers);
                }, project.getDisposed());
            });
        };

        project.getMessageBus().connect(toolWindow.getContentManager()).subscribe(FileEditorManagerListener.FILE_EDITOR_MANAGER, new FileEditorManagerListener() {
            @Override
            public void selectionChanged(@NotNull FileEditorManagerEvent event) {
                refresh.to(event.getNewFile());
            }
        });
        VirtualFile[] selected = FileEditorManager.getInstance(project).getSelectedFiles();
        refresh.to(selected.length > 0 ? selected[0] : null);
    }

    private interface Refresh {
        void to(VirtualFile file);
    }

    private static void show(DefaultListModel<Pointer> model, List<Pointer> pointers) {
        model.clear();
        model.addAll(pointers);
    }
}
