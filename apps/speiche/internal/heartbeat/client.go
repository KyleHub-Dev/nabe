package heartbeat

type Client struct {
	apiURL          string
	enrollmentToken string
}

func NewClient(apiURL string, enrollmentToken string) Client {
	return Client{apiURL: apiURL, enrollmentToken: enrollmentToken}
}

func (c Client) Status() string {
	if c.enrollmentToken == "" {
		return "heartbeat client ready without enrollment token"
	}

	return "heartbeat client ready for " + c.apiURL
}
