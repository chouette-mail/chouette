module Main exposing (main)

import Browser
import Element exposing (Element)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input
import Http
import Styles exposing (colors, defaultAttributes)


main =
    Browser.element
        { init = init
        , update = update
        , view = view
        , subscriptions = subscriptions
        }



-------------------------------------------------------------------------------
-- TYPES ----------------------------------------------------------------------
-------------------------------------------------------------------------------
-- LOG IN FORM ----------------------------------------------------------------


type alias LogInFormContent =
    { username : String
    , password : String
    }


defaultLogInFormContent =
    { username = "", password = "" }


type LogInFormMsg
    = LogInFormUsernameChanged String
    | LogInFormPasswordChanged String
    | LogInFormSubmitted



-- MAIN -----------------------------------------------------------------------


type alias PortalContent =
    { logInForm : LogInFormContent
    }


type Model
    = Portal PortalContent


type Msg
    = LogInMsg LogInFormMsg


defaultPortal =
    { logInForm = defaultLogInFormContent }


init : () -> ( Model, Cmd Msg )
init _ =
    ( Portal defaultPortal, Cmd.none )



-------------------------------------------------------------------------------
-- UPDATES --------------------------------------------------------------------
-------------------------------------------------------------------------------


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model ) of
        ( LogInMsg message, Portal content ) ->
            let
                ( newLogInForm, cmd ) =
                    updateLogInForm message content.logInForm
            in
            ( Portal { logInForm = newLogInForm }, Cmd.none )


updateLogInForm : LogInFormMsg -> LogInFormContent -> ( LogInFormContent, Cmd Msg )
updateLogInForm msg content =
    case msg of
        LogInFormUsernameChanged newUsername ->
            ( { content | username = newUsername }, Cmd.none )

        LogInFormPasswordChanged newPassword ->
            ( { content | password = newPassword }, Cmd.none )

        LogInFormSubmitted ->
            ( content, Cmd.none )



-------------------------------------------------------------------------------
-- SUBSCRIPTIONS --------------------------------------------------------------
-------------------------------------------------------------------------------


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-------------------------------------------------------------------------------
-- VIEWS ----------------------------------------------------------------------
-------------------------------------------------------------------------------


view model =
    case model of
        Portal content ->
            portalView content


portalView content =
    Element.layout defaultAttributes
        (Element.column
            (Element.width Element.fill :: defaultAttributes)
            [ header
            , portalContent content
            ]
        )


portalContent : PortalContent -> Element Msg
portalContent content =
    Element.row (Element.width Element.fill :: defaultAttributes)
        [ emptyColumn 1
        , portalPresentation
        , emptyColumn 2
        , portalForm content.logInForm
        , emptyColumn 1
        ]


portalForm : LogInFormContent -> Element Msg
portalForm content =
    Element.column [ Element.centerY, Element.spacing 10, Element.width <| Element.fillPortion 2 ]
        [ Styles.title "Log in"
        , Input.text Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Username")
            , onChange = LogInMsg << LogInFormUsernameChanged
            , placeholder = Nothing
            , text = content.username
            }
        , Input.currentPassword Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Password")
            , onChange = LogInMsg << LogInFormPasswordChanged
            , placeholder = Nothing
            , text = content.password
            , show = False
            }
        , Input.button
            (Element.centerX
                :: Border.solid
                :: Border.width 1
                :: Border.rounded 5
                :: Background.color Styles.colors.buttonNormal
                :: Element.padding 10
                :: Styles.defaultAttributes
            )
            { label = Element.text "Log in"
            , onPress = Just (LogInMsg LogInFormSubmitted)
            }
        ]


emptyColumn portion =
    Element.column [ Element.width <| Element.fillPortion portion ] []


portalPresentation =
    Element.column [ Element.centerY, Element.width <| Element.fillPortion 2 ]
        [ Styles.title "Welcome to chouette!"
        , Element.paragraph [ Font.center ]
            [ Element.text description ]
        ]


header =
    Element.row
        [ Element.width Element.fill
        , Element.alignTop
        , Element.padding 20
        , Font.color colors.contrastedText
        , Background.gradient { angle = pi / 2, steps = [ colors.accentLight, colors.accent ] }
        ]
        [ Element.column [ Element.width <| Element.fillPortion 3 ]
            [ Element.text "Logo" ]
        , Element.column [ Element.width <| Element.fillPortion 5 ]
            [ Element.text "Search bar" ]
        , Element.column [ Element.width <| Element.fillPortion 5 ]
            [ Element.text "Status bar" ]
        ]


description =
    """Chouette is a small and reliable open source web mail client written in Rust and Elm. It gives you all the keys you need to get started almost instantly, and remain the only person accessing and storing your emails."""
