module Component.Button exposing (floatingButton)

import Element exposing (Element, text, paddingXY)
import Element.Input as Input
import Element.Background as Background
import Element.Font as Font
import Element.Border as Border
import Styles exposing (colors, defaultAttributes)

floatingButton : String -> Maybe msg -> Element msg
floatingButton label msg =
    Input.button [ Element.centerX
                 , Background.color colors.accentLight
                 , Font.color colors.contrastedText
                 , paddingXY 20 10
                 , Border.rounded 3
                 , Border.shadow { offset = (0, 1)
                                 , size = 3
                                 , blur = 3
                                 , color = colors.shadow } ]
        { onPress = msg
        , label = text label
        }