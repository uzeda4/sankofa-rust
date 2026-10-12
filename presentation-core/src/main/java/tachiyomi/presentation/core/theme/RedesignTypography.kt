@file:OptIn(ExperimentalTextApi::class)

package tachiyomi.presentation.core.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.ExperimentalTextApi
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontVariation
import androidx.compose.ui.text.font.FontWeight
import tachiyomi.presentation.core.R

// Fonts are SIL OFL 1.1, license texts in docs/redesign/licenses. Both are variable fonts (API 26+ is our minSdk).

/** Display and title face. */
val BricolageGrotesque = FontFamily(
    variableFont(R.font.bricolage_grotesque, FontWeight.Medium),
    variableFont(R.font.bricolage_grotesque, FontWeight.Bold),
    variableFont(R.font.bricolage_grotesque, FontWeight.ExtraBold),
)

/** Body and label face. */
val DmSans = FontFamily(
    variableFont(R.font.dm_sans, FontWeight.Normal),
    variableFont(R.font.dm_sans, FontWeight.Medium),
    variableFont(R.font.dm_sans, FontWeight.Bold),
)

private fun variableFont(resId: Int, weight: FontWeight) = Font(
    resId = resId,
    weight = weight,
    variationSettings = FontVariation.Settings(FontVariation.weight(weight.weight)),
)

/**
 * Display, headline and large title styles use Bricolage Grotesque, everything else DM Sans.
 * Sizes, line heights and the original weights of the body styles are kept, so layouts do not move.
 */
fun Typography.withRedesignFonts(): Typography = copy(
    displayLarge = displayLarge.display(),
    displayMedium = displayMedium.display(),
    displaySmall = displaySmall.display(),
    headlineLarge = headlineLarge.display(),
    headlineMedium = headlineMedium.display(),
    headlineSmall = headlineSmall.display(),
    titleLarge = titleLarge.title(),
    titleMedium = titleMedium.title(),
    titleSmall = titleSmall.body(),
    bodyLarge = bodyLarge.body(),
    bodyMedium = bodyMedium.body(),
    bodySmall = bodySmall.body(),
    labelLarge = labelLarge.body(),
    labelMedium = labelMedium.body(),
    labelSmall = labelSmall.body(),
)

private fun TextStyle.display() = copy(fontFamily = BricolageGrotesque, fontWeight = FontWeight.ExtraBold)

private fun TextStyle.title() = copy(fontFamily = BricolageGrotesque, fontWeight = FontWeight.Bold)

private fun TextStyle.body() = copy(fontFamily = DmSans)
