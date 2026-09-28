// SPDX-License-Identifier: Apache-2.0
//
// One row of a pointer list: the path and name in bold, then the summary, then
// the warrant sentence when nobody accepted the document.

package ai.headwater.jetbrains;

import ai.headwater.jetbrains.Client.Pointer;
import com.intellij.ui.ColoredListCellRenderer;
import com.intellij.ui.SimpleTextAttributes;
import javax.swing.JList;
import org.jetbrains.annotations.NotNull;

final class PointerRenderer extends ColoredListCellRenderer<Pointer> {
    private static final long serialVersionUID = 1L;

    @Override
    protected void customizeCellRenderer(@NotNull JList<? extends Pointer> list, Pointer pointer, int index, boolean selected, boolean focused) {
        if (pointer == null) return;
        append(Client.label(pointer), SimpleTextAttributes.REGULAR_BOLD_ATTRIBUTES);
        if (pointer.summary() != null) append("  " + pointer.summary(), SimpleTextAttributes.REGULAR_ATTRIBUTES);
        if (pointer.asserted() != null) append("  asserted: " + pointer.asserted(), SimpleTextAttributes.GRAYED_ATTRIBUTES);
    }
}
