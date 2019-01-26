module Colors exposing (..)

import Color.Generator as Generator

import Color exposing (Color)
import Element

colorToElement : Color.Color -> Element.Color
colorToElement =
    colorToElementWithAlpha 1.0

colorToElementWithAlpha : Float -> Color.Color -> Element.Color
colorToElementWithAlpha alpha color =
    let (red, green, blue) = Color.toRGB color 
    in
    Element.fromRgb { red = red / 255.0
                    , green = green / 255.0
                    , blue = blue / 255.0
                    , alpha = alpha
                    }

background : Color 
background = Color.fromRGB ( 250, 250, 250 )

floatingBackground : Color
floatingBackground = Color.fromRGB ( 255, 255, 255 )

floatingShadow : Color
floatingShadow = Color.fromRGB ( 245, 245, 245 )

text : Color
text = Color.fromRGB ( 137, 143, 163 )

textContrasted : Color
textContrasted = Color.fromRGB ( 250, 250, 250 )

accentLight : Color
accentLight = Color.fromRGB (204, 43, 94)

accent : Color
accent = Color.fromRGB (204, 43, 94)

buttonNormal : Color
buttonNormal = Color.fromRGB (70, 188, 153)

shadow : Color
shadow = Color.fromRGB (229, 229, 229)

shadowFocus : Color
shadowFocus = Generator.tint 35 accentLight