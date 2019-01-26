module Main exposing (main)

import Browser
import Color
import Colors exposing (colorToElement)
import Component.Block exposing (floatingBlock, floatingBlockWithProperties)
import Component.Button exposing (floatingButton)
import Either exposing (Either)
import Element exposing (Element)
import Element.Background as Background
import Element.Border as Border
import Element.Font as Font
import Element.Input as Input
import Html
import Http
import Json.Decode exposing (Decoder, field, list, string)
import Spinner
import Styles exposing (colors, defaultAttributes, fontSizes)


main =
    Browser.element
        { init = init
        , update = update
        , view = view
        , subscriptions = \model -> Sub.map SpinnerMsg Spinner.subscription
        }



-------------------------------------------------------------------------------
-- TYPES ----------------------------------------------------------------------
-------------------------------------------------------------------------------
-- HELPER TYPES ---------------------------------------------------------------


type FormStatus
    = Idle
    | Submitted
    | Success
    | Failure


type alias Mailbox =
    { name : List String
    }


mailboxesDecoder =
    list (list (field "name" (list string)))


subjectsDecoder =
    list string



-- LOG IN FORM ----------------------------------------------------------------


type alias LogInFormContent =
    { username : String
    , password : String
    , status : FormStatus
    }


logInFormContentToUrlEncoded : LogInFormContent -> String
logInFormContentToUrlEncoded content =
    String.join "&"
        [ "username=" ++ content.username
        , "password=" ++ content.password
        ]


defaultLogInFormContent : LogInFormContent
defaultLogInFormContent =
    { username = "", password = "", status = Idle }


type LogInFormMsg
    = LogInFormUsernameChanged String
    | LogInFormPasswordChanged String
    | LogInFormSubmitted
    | LogInFormResponse (Result Http.Error String)



-- REGISTER FORM --------------------------------------------------------------


type alias RegisterFormContent =
    { username : String
    , email : String
    , password : String
    , status : FormStatus
    }


registerFormContentToUrlEncoded : RegisterFormContent -> String
registerFormContentToUrlEncoded content =
    String.join "&"
        [ "username=" ++ content.username
        , "email=" ++ content.email
        , "password=" ++ content.password
        ]


defaultRegisterFormContent : RegisterFormContent
defaultRegisterFormContent =
    { username = ""
    , email = ""
    , password = ""
    , status = Idle
    }


type RegisterFormMsg
    = RegisterFormUsernameChanged String
    | RegisterFormEmailChanged String
    | RegisterFormPasswordChanged String
    | RegisterFormSubmitted
    | RegisterFormResponse (Result Http.Error String)



-- NEW IMAP ACCOUNT FORM ------------------------------------------------------


type alias AddImapAccountFormContent =
    { server : String
    , username : String
    , password : String
    , status : FormStatus
    , tested : Bool
    }


addImapAccountFormContentToUrlEncoded : AddImapAccountFormContent -> String
addImapAccountFormContentToUrlEncoded content =
    String.join "&"
        [ "server=" ++ content.server
        , "username=" ++ content.username
        , "password=" ++ content.password
        ]


defaultAddImapAccountFormContent : AddImapAccountFormContent
defaultAddImapAccountFormContent =
    { server = ""
    , username = ""
    , password = ""
    , status = Idle
    , tested = False
    }


type AddImapAccountFormMsg
    = AddImapAccountFormServerChanged String
    | AddImapAccountFormUsernameChanged String
    | AddImapAccountFormPasswordChanged String
    | AddImapAccountFormTestSubmitted
    | AddImapAccountFormAddSubmitted
    | AddImapAccountFormTestResponse (Result Http.Error String)
    | AddImapAccountFormAddResponse (Result Http.Error String)



-- MAIN -----------------------------------------------------------------------


type PortalForm
    = LogInForm
    | RegisterForm


type HomePanel
    = HomePanelEmpty
    | HomePanelSubjects (List String)
    | HomePanelAddImapAccountForm


type alias PortalContent =
    { logInForm : LogInFormContent
    , registerForm : RegisterFormContent
    , form : PortalForm
    }


type alias HomeContent =
    { addImapAccountForm : AddImapAccountFormContent
    , mailboxes : List (List (List String))
    , panel : HomePanel
    }


type Page
    = Portal PortalContent
    | Home HomeContent


type alias Model =
    { page : Page
    , spinner : Spinner.Model
    }


type Msg
    = LogInFormMsg LogInFormMsg
    | RegisterFormMsg RegisterFormMsg
    | AddImapAccountFormMsg AddImapAccountFormMsg
    | GoToLogInFormMsg
    | GoToRegisterFormMsg
    | GoToPanelAddImapAccount
    | MailboxesMsg (Result Http.Error (List (List (List String))))
    | SubjectsMsg (Result Http.Error (List String))
    | SpinnerMsg Spinner.Msg


defaultPortalContent : PortalContent
defaultPortalContent =
    { logInForm = defaultLogInFormContent
    , registerForm = defaultRegisterFormContent
    , form = LogInForm
    }


init : () -> ( Model, Cmd Msg )
init _ =
    ( { page = Portal defaultPortalContent
      , spinner = Spinner.init
      }
    , Cmd.none
    )



-------------------------------------------------------------------------------
-- UPDATES --------------------------------------------------------------------
-------------------------------------------------------------------------------


update : Msg -> Model -> ( Model, Cmd Msg )
update msg model =
    case ( msg, model.page ) of
        ( LogInFormMsg message, Portal content ) ->
            let
                ( result, cmd ) =
                    updateLogInForm message content.logInForm
            in
            case result of
                Either.Left newLogInForm ->
                    ( { model | page = Portal { content | logInForm = newLogInForm } }, cmd )

                Either.Right m ->
                    ( m, cmd )

        ( RegisterFormMsg message, Portal content ) ->
            let
                ( newRegisterForm, cmd ) =
                    updateRegisterForm message content.registerForm
            in
            ( { model | page = Portal { content | registerForm = newRegisterForm } }, cmd )

        ( GoToLogInFormMsg, Portal content ) ->
            ( { model | page = Portal { content | form = LogInForm } }, Cmd.none )

        ( GoToRegisterFormMsg, Portal content ) ->
            ( { model | page = Portal { content | form = RegisterForm } }, Cmd.none )

        ( MailboxesMsg (Ok mailboxesContent), _ ) ->
            let
                newPage =
                    Home
                        { addImapAccountForm = defaultAddImapAccountFormContent
                        , mailboxes = mailboxesContent
                        , panel = HomePanelEmpty
                        }
            in
            case mailboxesContent of
                ((a :: _) :: _) :: _ ->
                    ( { model | page = newPage }, requestSubjects a )

                _ ->
                    ( { model | page = newPage }, Cmd.none )

        ( SubjectsMsg (Ok subjects), Home homeContent ) ->
            ( { model | page = Home { homeContent | panel = HomePanelSubjects subjects } }, Cmd.none )

        ( MailboxesMsg (Err mailboxesContent), Portal content ) ->
            let
                logInForm =
                    content.logInForm

                newLogInForm =
                    { logInForm | status = Failure }
            in
            ( { model | page = Portal { content | logInForm = newLogInForm } }, Cmd.none )

        ( GoToPanelAddImapAccount, Home content ) ->
            ( { model | page = Home { content | panel = HomePanelAddImapAccountForm } }, Cmd.none )

        ( AddImapAccountFormMsg message, Home content ) ->
            let
                ( result, cmd ) =
                    updateAddImapAccountForm message content.addImapAccountForm
            in
            case result of
                Either.Left newImapAccountForm ->
                    ( { model | page = Home { content | addImapAccountForm = newImapAccountForm } }, cmd )

                Either.Right newPanel ->
                    ( { model | page = Home { content | panel = newPanel } }, Cmd.none )

        ( SpinnerMsg message, Home content ) ->
            let
                spinnerModel =
                    Spinner.update message model.spinner
            in
            ( { model | spinner = spinnerModel }, Cmd.none )

        ( _, m ) ->
            ( model, Cmd.none )


{-|

    This function can either return a delta on the log in form content, or a
    completely new model in case the log in is succesful.

-}
updateLogInForm : LogInFormMsg -> LogInFormContent -> ( Either LogInFormContent Model, Cmd Msg )
updateLogInForm msg logInForm =
    case msg of
        LogInFormUsernameChanged newUsername ->
            ( Either.Left { logInForm | username = newUsername }, Cmd.none )

        LogInFormPasswordChanged newPassword ->
            ( Either.Left { logInForm | password = newPassword }, Cmd.none )

        LogInFormSubmitted ->
            ( Either.Left { logInForm | status = Submitted }, requestLogIn logInForm )

        LogInFormResponse (Ok resonse) ->
            ( Either.Left { logInForm | status = Success }, requestMailboxes )

        LogInFormResponse (Err resonse) ->
            ( Either.Left { logInForm | status = Failure }, Cmd.none )


updateRegisterForm : RegisterFormMsg -> RegisterFormContent -> ( RegisterFormContent, Cmd Msg )
updateRegisterForm msg registerForm =
    case msg of
        RegisterFormUsernameChanged newUsername ->
            ( { registerForm | username = newUsername }, Cmd.none )

        RegisterFormPasswordChanged newPassword ->
            ( { registerForm | password = newPassword }, Cmd.none )

        RegisterFormEmailChanged newEmail ->
            ( { registerForm | email = newEmail }, Cmd.none )

        RegisterFormSubmitted ->
            ( { registerForm | status = Submitted }, requestRegister registerForm )

        RegisterFormResponse (Ok response) ->
            ( { registerForm | status = Success }, Cmd.none )

        RegisterFormResponse (Err response) ->
            ( { registerForm | status = Failure }, Cmd.none )


updateAddImapAccountForm :
    AddImapAccountFormMsg
    -> AddImapAccountFormContent
    -> ( Either AddImapAccountFormContent HomePanel, Cmd Msg )
updateAddImapAccountForm msg addImapAccountForm =
    case msg of
        AddImapAccountFormUsernameChanged newUsername ->
            ( Either.Left { addImapAccountForm | username = newUsername, tested = False }, Cmd.none )

        AddImapAccountFormPasswordChanged newPassword ->
            ( Either.Left { addImapAccountForm | password = newPassword, tested = False }, Cmd.none )

        AddImapAccountFormServerChanged newServer ->
            ( Either.Left { addImapAccountForm | server = newServer, tested = False }, Cmd.none )

        AddImapAccountFormTestSubmitted ->
            ( Either.Left { addImapAccountForm | status = Submitted }
            , requestTestImapAccount addImapAccountForm
            )

        AddImapAccountFormAddSubmitted ->
            ( Either.Left { addImapAccountForm | status = Submitted }
            , requestAddImapAccount addImapAccountForm
            )

        AddImapAccountFormTestResponse (Ok response) ->
            ( Either.Left { addImapAccountForm | status = Success, tested = True }, Cmd.none )

        AddImapAccountFormTestResponse (Err response) ->
            ( Either.Left { addImapAccountForm | status = Failure, tested = False }, Cmd.none )

        AddImapAccountFormAddResponse (Ok response) ->
            ( Either.Right HomePanelEmpty, Cmd.none )

        AddImapAccountFormAddResponse (Err response) ->
            ( Either.Left { addImapAccountForm | status = Failure, tested = False }, Cmd.none )



-------------------------------------------------------------------------------
-- COMMANDS -------------------------------------------------------------------
-------------------------------------------------------------------------------


httpStringBody : String -> Http.Body
httpStringBody params =
    Http.stringBody "application/x-www-form-urlencoded" params


requestLogIn : LogInFormContent -> Cmd Msg
requestLogIn content =
    Http.post
        { url = "/api/login"
        , body = httpStringBody (logInFormContentToUrlEncoded content)
        , expect = Http.expectString (LogInFormMsg << LogInFormResponse)
        }


requestRegister : RegisterFormContent -> Cmd Msg
requestRegister content =
    Http.post
        { url = "/api/new-user"
        , body = httpStringBody (registerFormContentToUrlEncoded content)
        , expect = Http.expectString (RegisterFormMsg << RegisterFormResponse)
        }


requestAboutImapAccounts url msg content =
    Http.post
        { url = url
        , body = httpStringBody (addImapAccountFormContentToUrlEncoded content)
        , expect = Http.expectString msg
        }


requestTestImapAccount : AddImapAccountFormContent -> Cmd Msg
requestTestImapAccount =
    requestAboutImapAccounts
        "/api/test-imap-account"
        (AddImapAccountFormMsg << AddImapAccountFormTestResponse)


requestAddImapAccount : AddImapAccountFormContent -> Cmd Msg
requestAddImapAccount =
    requestAboutImapAccounts
        "/api/add-imap-account"
        (AddImapAccountFormMsg << AddImapAccountFormAddResponse)


requestMailboxes : Cmd Msg
requestMailboxes =
    Http.post
        { url = "/api/get-mailboxes"
        , body = Http.emptyBody
        , expect = Http.expectJson MailboxesMsg mailboxesDecoder
        }


requestSubjects : String -> Cmd Msg
requestSubjects inbox =
    Http.post
        { url = "/api/get-subjects"
        , body = httpStringBody ("inbox=" ++ inbox)
        , expect = Http.expectJson SubjectsMsg subjectsDecoder
        }



-------------------------------------------------------------------------------
-- SUBSCRIPTIONS --------------------------------------------------------------
-------------------------------------------------------------------------------


subscriptions : Model -> Sub Msg
subscriptions model =
    Sub.none



-------------------------------------------------------------------------------
-- VIEWS ----------------------------------------------------------------------
-------------------------------------------------------------------------------


view : Model -> Html.Html Msg
view model =
    case model.page of
        Portal content ->
            portalView content

        Home content ->
            homeView content model.spinner



-- HOME VIEWS -----------------------------------------------------------------


homeView : HomeContent -> Spinner.Model -> Html.Html Msg
homeView homeContent spinner =
    Element.layout defaultAttributes
        (Element.column
            [ Element.width Element.fill ]
            [ header
            , Element.column
                [ Element.padding 40
                , Element.width Element.fill
                ]
                [ Element.row
                    [ Element.width Element.fill
                    , Element.height Element.fill
                    , Element.spacing 20
                    ]
                    [ leftMenu homeContent
                    , homePanel homeContent spinner
                    ]
                ]
            ]
        )


homePanel : HomeContent -> Spinner.Model -> Element Msg
homePanel content spinner =
    let
        ( parseContent, showIntoBlock ) =
            case content.panel of
                HomePanelEmpty ->
                    ( [ Element.html (Spinner.view Spinner.defaultConfig spinner) ], False )

                HomePanelSubjects subjects ->
                    ( List.map Element.text subjects, True )

                HomePanelAddImapAccountForm ->
                    ( [ homePanelAddImapAccountForm content.addImapAccountForm ], True )
    in
    if showIntoBlock then
        floatingBlockWithProperties
            [ Element.width <| Element.fillPortion 8
            , Element.alignTop
            ]
            parseContent

    else
        Element.column
            [ Element.width <| Element.fillPortion 8
            , Element.height Element.fill
            , Element.centerX
            , Element.centerY
            ]
            parseContent


homePanelAddImapAccountForm : AddImapAccountFormContent -> Element Msg
homePanelAddImapAccountForm content =
    let
        testText =
            case content.status of
                Idle ->
                    Element.text "Test IMAP account"

                Submitted ->
                    Element.text "Testing IMAP account..."

                Success ->
                    Element.text "IMAP account works! Click to try again."

                Failure ->
                    Element.text "Couldn't connect to IMAP server. Click to try again."

        addText =
            if content.status == Success then
                Element.text "Add IMAP account"

            else
                Element.text "Add IMAP account (please test before adding)"

        testMessage =
            if content.status /= Submitted then
                Just (AddImapAccountFormMsg AddImapAccountFormTestSubmitted)

            else
                Nothing

        addMessage =
            if content.status == Success then
                Just (AddImapAccountFormMsg AddImapAccountFormAddSubmitted)

            else
                Nothing
    in
    Element.column
        [ Element.width Element.fill
        , Element.centerX
        , Element.centerY
        , Element.spacing 10
        ]
        [ Styles.title "Add a new IMAP account"
        , Input.text Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Server")
            , onChange = AddImapAccountFormMsg << AddImapAccountFormServerChanged
            , placeholder = Nothing
            , text = content.server
            }
        , Input.text Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Username (if you don't know it, it's probably your e-mail address)")
            , onChange = AddImapAccountFormMsg << AddImapAccountFormUsernameChanged
            , placeholder = Nothing
            , text = content.username
            }
        , Input.currentPassword Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Password")
            , onChange = AddImapAccountFormMsg << AddImapAccountFormPasswordChanged
            , placeholder = Nothing
            , text = content.password
            , show = False
            }
        , Element.row (Element.centerX :: Element.spacing 10 :: defaultAttributes)
            [ Input.button
                (Element.centerX
                    :: Border.solid
                    :: Border.width 1
                    :: Border.rounded 5
                    :: Background.color Styles.colors.buttonNormal
                    :: Element.padding 10
                    :: Styles.defaultAttributes
                )
                { label = testText
                , onPress = testMessage
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
                { label = addText
                , onPress = addMessage
                }
            ]
        ]


leftMenu : HomeContent -> Element Msg
leftMenu homeContent =
    let
        mailboxes =
            List.concat homeContent.mailboxes

        names =
            List.map (String.join "/") mailboxes

        mailboxItems =
            List.map (menuItem Nothing) names
    in
    floatingBlockWithProperties
        [ Element.width <| Element.fillPortion 2
        , Element.alignTop
        , Element.padding 0
        ]
        (menuItem (Just GoToPanelAddImapAccount) "Add new IMAP account"
            :: mailboxItems
        )


menuItem : Maybe Msg -> String -> Element Msg
menuItem message linkText =
    let
        label =
            Element.row
                [ Element.width Element.fill
                , Element.padding 20
                , Border.color <| colorToElement Colors.itemHoverBorder
                , Border.widthEach
                    { bottom = 0
                    , left = 4
                    , top = 0
                    , right = 0
                    }
                , Element.mouseOver
                    [ Background.color <| colorToElement Colors.itemHoverBackground
                    , Font.color <| colorToElement Colors.itemHoverText
                    ]
                ]
                [ Element.text linkText ]
    in
    Input.button [ Element.width Element.fill, Element.height Element.fill ]
        { onPress = message
        , label = label
        }



-- PORTAL VIEWS ---------------------------------------------------------------


portalView : PortalContent -> Html.Html Msg
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
    Element.row
        [ Element.width Element.fill
        , Element.padding 20
        , Element.alignTop
        ]
        [ emptyColumn 1
        , portalPresentation
        , emptyColumn 1
        , portalForm content
        , emptyColumn 1
        ]


portalForm : PortalContent -> Element Msg
portalForm content =
    case content.form of
        LogInForm ->
            portalLogInForm content.logInForm

        RegisterForm ->
            portalRegisterForm content.registerForm


portalLogInForm : LogInFormContent -> Element Msg
portalLogInForm content =
    let
        text =
            case content.status of
                Idle ->
                    "Log in"

                Submitted ->
                    "Logging in..."

                Success ->
                    "Logged in!"

                Failure ->
                    "An error occured :'( Click to retry"

        message =
            if content.status == Idle || content.status == Failure then
                Just (LogInFormMsg LogInFormSubmitted)

            else
                Nothing
    in
    floatingBlockWithProperties
        [ Element.width <| Element.fillPortion 2
        , Element.spacing 12
        ]
        [ Styles.title "Log in"
        , Input.text []
            { label =
                Input.labelAbove [ Element.centerY, Element.padding 5 ]
                    (Element.text "Username")
            , onChange = LogInFormMsg << LogInFormUsernameChanged
            , placeholder = Nothing
            , text = content.username
            }
        , Input.currentPassword []
            { label =
                Input.labelAbove [ Element.centerY, Element.padding 5 ]
                    (Element.text "Password")
            , onChange = LogInFormMsg << LogInFormPasswordChanged
            , placeholder = Nothing
            , text = content.password
            , show = False
            }
        , floatingButton text message
        , Input.button [ Font.size fontSizes.small ]
            { label = Element.text "Not registered yet ? Click here to register."
            , onPress = Just GoToRegisterFormMsg
            }
        ]


portalRegisterForm : RegisterFormContent -> Element Msg
portalRegisterForm content =
    let
        text =
            case content.status of
                Idle ->
                    Element.text "Register"

                Submitted ->
                    Element.text "Registering..."

                Success ->
                    Element.text "Succesfully registered!"

                Failure ->
                    Element.text "An error occured :'("

        message =
            if content.status == Idle then
                Just (RegisterFormMsg RegisterFormSubmitted)

            else
                Nothing
    in
    Element.column [ Element.centerY, Element.spacing 10, Element.width <| Element.fillPortion 2 ]
        [ Styles.title "Log in"
        , Input.text Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Username")
            , onChange = RegisterFormMsg << RegisterFormUsernameChanged
            , placeholder = Nothing
            , text = content.username
            }
        , Input.text Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Email")
            , onChange = RegisterFormMsg << RegisterFormEmailChanged
            , placeholder = Nothing
            , text = content.email
            }
        , Input.currentPassword Styles.defaultAttributes
            { label =
                Input.labelAbove (Element.centerY :: Element.padding 5 :: Styles.defaultAttributes)
                    (Element.text "Password")
            , onChange = RegisterFormMsg << RegisterFormPasswordChanged
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
            { label = text
            , onPress = message
            }
        , Input.button Styles.defaultAttributes
            { label = Element.text "Already registered ? Click here to log in."
            , onPress = Just GoToLogInFormMsg
            }
        ]


portalPresentation : Element msg
portalPresentation =
    floatingBlockWithProperties [ Element.width <| Element.fillPortion 2 ]
        [ Styles.title "Welcome to chouette!"
        , Element.paragraph [ Font.center ]
            [ Element.text description ]
        ]



-- UTILS ----------------------------------------------------------------------


emptyColumn : Int -> Element Msg
emptyColumn portion =
    Element.column [ Element.width <| Element.fillPortion portion ] []


header : Element msg
header =
    Element.row
        [ Element.width Element.fill
        , Element.alignTop
        , Element.padding 20
        , Font.color colors.contrastedText
        , Background.gradient { angle = pi / 2, steps = [ colors.accentLight, colors.accent ] }
        ]
        [ Element.column [ Element.width <| Element.fillPortion 3 ]
            [ Element.text "chouette" ]
        , Element.column [ Element.width <| Element.fillPortion 5 ]
            [ Element.text "Search bar" ]
        , Element.column [ Element.width <| Element.fillPortion 5 ]
            [ Element.text "Status bar" ]
        ]


description : String
description =
    """Chouette is a small and reliable open source web mail client written in Rust and Elm. It gives you all the keys you need to get started almost instantly, and remain the only person accessing and storing your emails."""
