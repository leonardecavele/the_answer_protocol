package error

const (
	NoError = 0

	NameInUseError = 201
	NoContentError = 204
	NoExitError    = 301

	AlreadyConnectedError = 400
	NotConnectedError     = 400
	InvalidUsernameError  = 400
	RoomFullError         = 400
	GroupFullError        = 400
	EmptyCommandError     = 400
	CommandNotFoundError  = 400
	InvalidArgumentsError = 400
	InvalidScopeError     = 400

	NotInGroupError     = 401
	AlreadyInGroupError = 402

	NoSuchUserError     = 403
	NotInvitedError     = 403
	NotGroupLeaderError = 403

	ItemNotFoundError       = 404
	ItemNotInInventoryError = 404
	NpcNotFoundError        = 404
	GroupNotFoundError      = 404
	NoSuchGroupError        = 404

	PlayerNotFoundError        = 405
	NpcNotHostileError         = 405
	NoQuestAvailableError      = 406
	NpcNotInRoomError          = 407
	NotInSameRoomError         = 407
	NpcInCombatError           = 408
	ActionAlreadyTakenError    = 409
	PlayerAlreadyInCombatError = 410
	PlayerNotInCombatError     = 411
	FileNotFoundError          = 412
	RoomNotFoundError          = 413

	ConnectionFailedError  = 900
	SendFailedError        = 901
	GameServerTimeoutError = 902

	InvalidGroupCommandError = 997
	InvalidQuestionError     = 998
	InvalidCommandError      = 999
	UnknownError             = 999
)
