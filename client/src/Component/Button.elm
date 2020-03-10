module Component.Button exposing (floatingButton)

import Colors exposing (..)
import Element exposing (Element, focused, paddingXY)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input


floatingButton : String -> Maybe msg -> Element msg
floatingButton label msg =
    Input.button
        [ Element.centerX
        , Background.color <| colorToElement Colors.accentLight
        , Font.color <| colorToElement Colors.textContrasted
        , paddingXY 20 10
        , Border.rounded 3
        , Border.shadow
            { offset = ( 0, 1 )
            , size = 3
            , blur = 3
            , color = colorToElement Colors.shadow
            }
        , focused
            [ Border.shadow
                { offset = ( 0, 0 )
                , size = 4
                , blur = 0
                , color = colorToElement Colors.shadowFocus
                }
            ]
        ]
        { onPress = msg
        , label = Element.text label
        }
