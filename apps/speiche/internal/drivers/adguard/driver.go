package adguard

type Driver struct {
	baseURL string
}

func NewDriver(baseURL string) Driver {
	return Driver{baseURL: baseURL}
}

func (d Driver) Status() string {
	return "local AdGuard driver placeholder: " + d.baseURL
}
