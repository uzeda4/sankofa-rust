package eu.kanade.presentation.more

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.em
import tachiyomi.i18n.MR
import tachiyomi.i18n.kmk.KMR
import tachiyomi.presentation.core.components.BrandMark
import tachiyomi.presentation.core.i18n.stringResource
import tachiyomi.presentation.core.util.secondaryItemAlpha

// KMK -->
@Composable
fun LogoHeader() {
    Column(
        modifier = Modifier.fillMaxWidth(),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        BrandMark(
            size = 88.dp,
            modifier = Modifier.padding(top = 48.dp),
        )
        Text(
            text = stringResource(MR.strings.app_name),
            modifier = Modifier.padding(top = 12.dp),
            style = MaterialTheme.typography.headlineMedium,
            fontWeight = FontWeight.Bold,
            letterSpacing = (-0.01).em,
        )
        Text(
            text = stringResource(KMR.strings.redesign_eyebrow_rediscover),
            modifier = Modifier
                .padding(top = 2.dp, bottom = 40.dp)
                .secondaryItemAlpha(),
            style = MaterialTheme.typography.bodyMedium,
        )

        HorizontalDivider()
    }
}
// KMK <--
