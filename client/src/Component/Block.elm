module Component.Block exposing (floatingBlock)

import Element
import Colors exposing (colorToElement)
import Element.Border as Border
import Element.Background as Background

floatingBlock content = 
    Element.column [ Element.centerX
                   , Element.centerY
                   , Element.spacing 12
                   , Element.width <| Element.fillPortion 2
                   , Background.color <| colorToElement Colors.floatingBackground
                   , Element.padding 20
                   , Border.shadow { offset = (0, 0)
                                   , color = colorToElement Colors.floatingShadow
                                   , blur = 5
                                   , size = 5 } ]
                    content