// SPDX-License-Identifier: Apache-2.0
//
// Intent time: the action *Headwater: What governs this task?* asks for a task,
// runs `Client.route` on a pooled thread and lists the pointers it returns, as a
// path, a name and a summary, never the content of a document. Choosing one
// opens it. It shows nothing when there are no pointers.

package ai.headwater.jetbrains;

import ai.headwater.jetbrains.Client.Answer;
import ai.headwater.jetbrains.Client.Options;
import ai.headwater.jetbrains.Client.Pointer;
import com.intellij.openapi.actionSystem.ActionUpdateThread;
import com.intellij.openapi.actionSystem.AnAction;
import com.intellij.openapi.actionSystem.AnActionEvent;
import com.intellij.openapi.application.ApplicationManager;
import com.intellij.openapi.project.DumbAware;
import com.intellij.openapi.project.Project;
import com.intellij.openapi.ui.Messages;
import com.intellij.openapi.ui.popup.JBPopupFactory;
import org.jetbrains.annotations.NotNull;

public final class WhatGovernsAction extends AnAction implements DumbAware {
    @Override
    public @NotNull ActionUpdateThread getActionUpdateThread() {
        return ActionUpdateThread.BGT;
    }

    @Override
    public void update(@NotNull AnActionEvent event) {
        event.getPresentation().setEnabled(Headwater.root(event.getProject()) != null);
    }

    @Override
    public void actionPerformed(@NotNull AnActionEvent event) {
        Project project = event.getProject();
        Options options = Headwater.options(project);
        if (options == null) return;
        String task = Messages.showInputDialog(project, "What are you about to do?", "Headwater: What Governs This Task?", null);
        if (task == null || task.isBlank()) return;
        ApplicationManager.getApplication().executeOnPooledThread(() -> {
            Answer answer = Client.route(task, options);
            if (answer.pointers().isEmpty()) return;
            ApplicationManager.getApplication().invokeLater(() -> show(project, answer), project.getDisposed());
        });
    }

    private static void show(Project project, Answer answer) {
        String note = Client.withheldNote(answer);
        JBPopupFactory.getInstance()
                .createPopupChooserBuilder(answer.pointers())
                .setTitle("Documents that govern this task")
                .setAdText(note)
                .setRenderer(new PointerRenderer())
                .setItemChosenCallback((Pointer chosen) -> Headwater.open(project, chosen))
                .createPopup()
                .showCenteredInCurrentWindow(project);
    }
}
