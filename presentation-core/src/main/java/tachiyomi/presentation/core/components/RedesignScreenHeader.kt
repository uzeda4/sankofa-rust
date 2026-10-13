package tachiyomi.presentation.core.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em

/**
 * Large screen title of the redesign: small eyebrow above a display-style title.
 * Screens show it as the first item of their list instead of the title in the app bar.
 */
@Composable
fun RedesignScreenHeader(
    eyebrow: String,
    title: String,
    modifier: Modifier = Modifier,
) {
    Column(
        modifier = modifier
            .fillMaxWidth()
            .padding(horizontal = 16.dp, vertical = 8.dp),
        verticalArrangement = Arrangement.spacedBy(2.dp),
    ) {
        SectionLabel(text = eyebrow)
        Text(
            text = title,
            style = MaterialTheme.typography.displaySmall,
            letterSpacing = (-0.02).em,
            maxLines = 1,
            overflow = TextOverflow.Ellipsis,
        )
    }
}
