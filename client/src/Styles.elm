module Styles exposing (colors, defaultAttributes, title)

import Element exposing (Element, rgb255)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font


defaultAttributes : List (Element.Attribute msg)
defaultAttributes =
    [ Background.color colors.background
    , Font.family [ Font.typeface "Roboto Condensed", Font.sansSerif ]
    , Font.size 18
    , Font.color colors.text
    ]


colors =
    { background = rgb255 250 250 250
    , text = rgb255 137 143 163
    , contrastedText = rgb255 250 250 250
    , accentLight = rgb255 204 43 94
    , accent = rgb255 117 58 136
    , buttonNormal = rgb255 70 188 153

    -- , accentLight = rgb255 250 208 196
    -- , accent = rgb255 255 154 158}
    }


title titleText =
    Element.row
        [ Element.padding 30
        , Element.centerX
        , Font.size 32
        , Font.color colors.accentLight
        ]
        [ Element.text titleText ]
