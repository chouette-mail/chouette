module Component.Block exposing (floatingBlock, floatingBlockWithProperties)

import Colors exposing (colorToElement)
import Element
import Element.Background as Background
import Element.Border as Border


floatingBlock =
    floatingBlockWithProperties []


floatingBlockWithProperties properties content =
    Element.column
        (List.concat
            [ [ Element.centerX
              , Element.centerY
              , Background.color <| colorToElement Colors.floatingBackground
              , Element.padding 20
              , Border.shadow
                    { offset = ( 0, 0 )
                    , color = colorToElement Colors.floatingShadow
                    , blur = 5
                    , size = 5
                    }
              ]
            , properties
            ]
        )
        content
